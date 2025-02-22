use thiserror::Error;

#[cfg(feature = "supranational")]
use crate::supranational::errors::BlstError;

#[derive(Error, Debug)]
pub enum BLSError {
    #[cfg(feature = "supranational")]
    #[error("blst error: {0}")]
    BlstError(#[from] BlstError),
}
