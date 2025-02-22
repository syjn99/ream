use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZkcryptoError {
    #[error("Invalid infinity point: {group}")]
    InvalidInfinityPoint {
        group: String, // G1 or G2
    },
}
