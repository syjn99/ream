use crate::{errors::BLSError, AggregatePubKey, PubKey};

pub trait Aggregatable {
    type Error;
    fn aggregate(pubkeys: &[&PubKey]) -> Result<AggregatePubKey, Self::Error>;
}

pub trait ZkcryptoAggregatable: Aggregatable<Error = BLSError> {}
pub trait SupranationalAggregatable: Aggregatable<Error = anyhow::Error> {}

pub trait Verifiable {
    type Error;

    /// Verifies a BLS signature against a public key and message.
    ///
    /// # Arguments
    /// * `pubkey` - The public key to verify against
    /// * `message` - The message that was signed
    ///
    /// # Returns
    /// * `Result<bool, BLSError>` - Ok(true) if the signature is valid, Ok(false) if verification
    ///   fails, or Err if there are issues with signature or public key bytes
    fn verify(&self, pubkey: &PubKey, message: &[u8]) -> Result<bool, Self::Error>;

    /// Verifies the signature against a message using an aggregate of multiple public keys
    ///
    /// # Arguments
    /// * `pubkeys` - Collection of public key references to verify against
    /// * `message` - Message that was signed
    ///
    /// # Returns
    /// * `Result<bool, BLSError>` - Ok(true) if the signature is valid for the aggregate
    ///   verification, Ok(false) if verification fails, or Err if there are issues with signature
    ///   or public key bytes
    fn fast_aggregate_verify<'a, P>(&self, pubkeys: P, message: &[u8]) -> Result<bool, Self::Error>
    where
        P: AsRef<[&'a PubKey]>;
}

pub trait ZkcryptoVerifiable: Verifiable<Error = BLSError> {}
pub trait SupranationalVerifiable: Verifiable<Error = BLSError> {}
