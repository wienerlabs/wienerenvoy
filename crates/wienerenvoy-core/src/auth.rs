//! Bearer-token auth store.
//!
//! The token is the security boundary for every privileged action, so it is
//! generated from the OS CSPRNG, wrapped in `SecretString` so it never lands in
//! `Debug`/logs, and compared in constant time. The roadmap swaps this for
//! Tailscale LocalAPI `whois` identity, which is why callers go through the
//! `Authenticator` trait rather than touching `AuthStore` directly.

use std::fs;
use std::path::Path;

use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use subtle::ConstantTimeEq;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("token i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("token file {path} has insecure permissions {mode:o}; expected 0640 or stricter")]
    InsecurePermissions { path: String, mode: u32 },
    #[error("token file is empty")]
    Empty,
}

/// Holds the daemon's bearer token and verifies presented tokens.
pub struct AuthStore {
    secret: SecretString,
}

impl AuthStore {
    /// Generate a fresh random bearer token: 32 bytes of CSPRNG output, hex
    /// encoded to a 64-char ASCII string.
    #[must_use]
    pub fn generate_token() -> SecretString {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        use std::fmt::Write as _;
        let mut hex = String::with_capacity(64);
        for b in bytes {
            let _ = write!(hex, "{b:02x}");
        }
        SecretString::from(hex)
    }

    #[must_use]
    pub fn from_secret(secret: SecretString) -> Self {
        Self { secret }
    }

    /// Verify a presented bearer token against the stored one in constant time.
    /// Never use `==` here: a timing side channel leaks the token byte by byte.
    #[must_use]
    pub fn verify(&self, presented: &str) -> bool {
        let expected = self.secret.expose_secret();
        expected.as_bytes().ct_eq(presented.as_bytes()).into()
    }

    /// Expose the raw token. Use only at the trust boundary (writing the token
    /// file, `wenvoy token show`). Never log the result.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.secret.expose_secret()
    }
}

/// Read a token from a file, refusing it if the file permissions are too open.
/// On a root daemon a group/world readable token is a real leak, so we fail
/// loudly rather than trust it.
#[cfg(unix)]
pub fn read_token_file(path: &Path) -> Result<SecretString, AuthError> {
    use std::os::unix::fs::PermissionsExt;

    let meta = fs::metadata(path)?;
    let mode = meta.permissions().mode() & 0o777;
    // Reject if group-write or any world bits are set. 0o640 (rw-r-----) is the
    // most permissive we accept (owner root, group admin read).
    if mode & 0o037 & !0o040 != 0 {
        return Err(AuthError::InsecurePermissions {
            path: path.display().to_string(),
            mode,
        });
    }
    let raw = fs::read_to_string(path)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AuthError::Empty);
    }
    Ok(SecretString::from(trimmed.to_string()))
}

#[cfg(not(unix))]
pub fn read_token_file(path: &Path) -> Result<SecretString, AuthError> {
    let raw = fs::read_to_string(path)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AuthError::Empty);
    }
    Ok(SecretString::from(trimmed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_is_64_hex_chars() {
        let t = AuthStore::generate_token();
        let s = t.expose_secret();
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn verify_accepts_matching_token() {
        let secret = SecretString::from("abc123".to_string());
        let store = AuthStore::from_secret(secret);
        assert!(store.verify("abc123"));
    }

    #[test]
    fn verify_rejects_wrong_token() {
        let store = AuthStore::from_secret(SecretString::from("abc123".to_string()));
        assert!(!store.verify("abc124"));
        assert!(!store.verify("abc1234"));
        assert!(!store.verify(""));
    }

    #[test]
    fn two_generated_tokens_differ() {
        let a = AuthStore::generate_token();
        let b = AuthStore::generate_token();
        assert_ne!(a.expose_secret(), b.expose_secret());
    }
}
