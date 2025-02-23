use blst::BLST_ERROR;
use thiserror::Error;

/// Wrapper for the errors returned by the blst library
#[derive(Error, Debug, PartialEq)]
pub enum BlstError {
    #[error("Invalid BLS public key encoding")]
    BadEncoding,
    #[error("BLS point is not on curve")]
    PointNotOnCurve,
    #[error("BLS point is not in the correct group")]
    PointNotInGroup,
    #[error("BLS aggregate type mismatch")]
    AggrTypeMismatch,
    #[error("BLS verification failed")]
    VerificationFailed,
    #[error("BLS public key is infinity")]
    PublicKeyIsInfinity,
    #[error("Invalid BLS scalar value")]
    BadScalar,
    #[error("Unknown BLS error: {0:?}")]
    Unknown(BLST_ERROR),
}

impl From<BLST_ERROR> for BlstError {
    fn from(error: BLST_ERROR) -> Self {
        match error {
            BLST_ERROR::BLST_BAD_ENCODING => BlstError::BadEncoding,
            BLST_ERROR::BLST_POINT_NOT_ON_CURVE => BlstError::PointNotOnCurve,
            BLST_ERROR::BLST_POINT_NOT_IN_GROUP => BlstError::PointNotInGroup,
            BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => BlstError::AggrTypeMismatch,
            BLST_ERROR::BLST_VERIFY_FAIL => BlstError::VerificationFailed,
            BLST_ERROR::BLST_PK_IS_INFINITY => BlstError::PublicKeyIsInfinity,
            BLST_ERROR::BLST_BAD_SCALAR => BlstError::BadScalar,
            // SAFETY: BLST_SUCCESS should never be passed to this error conversion.
            // It is used only for comparison in verification methods and represents
            // a successful operation, not an error condition.
            BLST_ERROR::BLST_SUCCESS => unreachable!("Success is not an error"),
        }
    }
}
