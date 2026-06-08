//! Typed `501 Not Implemented` placeholder for routes that land in M1. The
//! dashboard reads the `error` code and greys out the corresponding controls.

use crate::error::ApiError;

pub async fn not_implemented() -> ApiError {
    ApiError::NotImplemented("available in M1 (power and telemetry)".to_string())
}
