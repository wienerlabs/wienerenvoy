//! Errors from Docker operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DockerError {
    #[error("Docker is not available")]
    Unavailable,
    #[error("docker error: {0}")]
    Bollard(#[from] bollard::errors::Error),
}
