use alloy_primitives::hex;
use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    pairing, G1Affine, G2Affine, G2Projective,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssz::Encode;
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, FixedVector};
use tree_hash_derive::TreeHash;

use super::pubkey::PubKey;
use crate::{constants::DST, errors::BLSError, AggregatePubKey};

#[derive(Debug, PartialEq, Clone, Encode, Decode, TreeHash, Default)]
pub struct BlsSignature {
    pub inner: FixedVector<u8, typenum::U96>,
}

impl TryFrom<BlsSignature> for G2Affine {
    type Error = BLSError;

    fn try_from(value: BlsSignature) -> Result<Self, Self::Error> {
        match G2Affine::from_compressed(
            &value
                .to_bytes()
                .try_into()
                .map_err(|_| BLSError::InvalidByteLength)?,
        )
        .into_option()
        {
            Some(point) => Ok(point),
            None => Err(BLSError::InvalidSignature),
        }
    }
}

impl Serialize for BlsSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = hex::encode(self.inner.as_ssz_bytes());
        serializer.serialize_str(&val)
    }
}

impl<'de> Deserialize<'de> for BlsSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result: String = Deserialize::deserialize(deserializer)?;
        let result = hex::decode(&result).map_err(serde::de::Error::custom)?;
        let key = FixedVector::from(result);
        Ok(Self { inner: key })
    }
}

impl BlsSignature {
    pub fn to_bytes(&self) -> &[u8] {
        self.inner.iter().as_slice()
    }

    pub fn infinity() -> Self {
        Self {
            inner: FixedVector::from(G2Affine::identity().to_compressed().to_vec()),
        }
    }

    /// Verifies a BLS signature against a public key and message.
    ///
    /// # Arguments
    /// * `pubkey` - The public key to verify against
    /// * `message` - The message that was signed
    ///
    /// # Returns
    /// * `Result<bool, BLSError>` - Ok(true) if the signature is valid, Ok(false) if verification
    ///   fails, or Err if there are issues with signature or public key bytes
    pub fn verify(&self, pubkey: &PubKey, message: &[u8]) -> Result<bool, BLSError> {
        let h = <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
            [message],
            DST,
        );

        let gt1 = pairing(&G1Affine::try_from(pubkey.clone())?, &G2Affine::from(h));
        let gt2 = pairing(&G1Affine::generator(), &G2Affine::try_from(self.clone())?);

        Ok(gt1 == gt2)
    }

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
    pub fn fast_aggregate_verify<'a, P>(&self, pubkeys: P, message: &[u8]) -> Result<bool, BLSError>
    where
        P: AsRef<[&'a PubKey]>,
    {
        let agg_pubkey = AggregatePubKey::aggregate(pubkeys.as_ref())?;

        let h = <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
            [message],
            DST,
        );

        let gt1 = pairing(
            &G1Affine::try_from(agg_pubkey.to_pubkey())?,
            &G2Affine::from(h),
        );
        let gt2 = pairing(&G1Affine::generator(), &G2Affine::try_from(self.clone())?);

        Ok(gt1 == gt2)
    }
}
