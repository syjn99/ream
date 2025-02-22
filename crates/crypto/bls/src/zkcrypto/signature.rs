use alloy_primitives::hex;
use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    pairing, G1Affine, G1Projective, G2Affine, G2Projective,
};
use serde::{de::Error as SerdeError, Deserialize, Deserializer, Serialize, Serializer};
use ssz::{Decode, Encode};
use tree_hash::{merkle_root, Hash256, PackedEncoding, TreeHash, TreeHashType};

use super::pubkey::PubKey;
use crate::{constants::DST, errors::BLSError};

#[derive(Debug, PartialEq, Clone)]
pub struct BlsSignature {
    pub inner: G2Projective,
}

impl Encode for BlsSignature {
    fn is_ssz_fixed_len() -> bool {
        true
    }
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&G2Affine::from(&self.inner).to_compressed());
    }
    fn ssz_bytes_len(&self) -> usize {
        96
    }
    fn ssz_fixed_len() -> usize {
        96
    }
}

impl Decode for BlsSignature {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        96
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let point = match G2Affine::from_compressed(bytes.try_into().map_err(|_| {
            ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 96,
            }
        })?)
        .into_option()
        {
            Some(p) => p,
            None => {
                return Err(ssz::DecodeError::BytesInvalid(
                    "Invalid signature dd".to_string(),
                ));
            }
        };

        Ok(Self {
            inner: point.into(),
        })
    }
}

impl Serialize for BlsSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&G2Affine::from(&self.inner).to_compressed())
    }
}

impl<'de> Deserialize<'de> for BlsSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result: String = Deserialize::deserialize(deserializer)?;
        let result = hex::decode(&result).map_err(serde::de::Error::custom)?;
        let mut signature = [0u8; 96];
        signature.copy_from_slice(&result);

        let point = match G2Affine::from_compressed(&signature).into_option() {
            Some(p) => p,
            None => {
                return Err(SerdeError::custom("Invalid signature"));
            }
        };

        Ok(Self {
            inner: point.into(),
        })
    }
}

impl TreeHash for BlsSignature {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        PackedEncoding::from_vec(G2Affine::from(&self.inner).to_compressed().to_vec())
    }

    fn tree_hash_packing_factor() -> usize {
        1
    }

    fn tree_hash_root(&self) -> Hash256 {
        merkle_root(&G2Affine::from(&self.inner).to_compressed(), 1)
    }
}

impl BlsSignature {
    pub fn to_bytes(&self) -> [u8; 96] {
        G2Affine::from(&self.inner).to_compressed()
    }

    pub fn infinity() -> Self {
        Self {
            inner: G2Projective::identity(),
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

        let gt1 = pairing(&G1Affine::from(pubkey.inner), &G2Affine::from(h));
        let gt2 = pairing(&G1Affine::generator(), &G2Affine::from(self.inner));

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
        let agg_pks_point = pubkeys
            .as_ref()
            .iter()
            .fold(G1Projective::identity(), |acc, pubkey| {
                acc.add(&pubkey.inner)
            });

        let h = <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
            [message],
            DST,
        );

        let gt1 = pairing(&G1Affine::from(agg_pks_point), &G2Affine::from(h));
        let gt2 = pairing(&G1Affine::generator(), &G2Affine::from(self.inner));

        Ok(gt1 == gt2)
    }
}
