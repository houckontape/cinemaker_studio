use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonError {
    #[error("EDSDK error code: {0}")]
    EDSDKError(i32),

    #[error("No camera found")]
    NoCamera,

    #[error("Invalid index")]
    InvalidIndex,

    #[error("FFI null pointer")]
    NullPointer,

    #[error("LiveView not available")]
    LiveViewUnavailable,

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CanonError>;

pub fn check_edsdk(code: i32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(CanonError::EDSDKError(code))
    }
}
