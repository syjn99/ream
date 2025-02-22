use alloy_primitives::hex;
use blst::{min_pk::Signature as BlstSignature, BLST_ERROR};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssz::{Decode, Encode};
use tree_hash::{merkle_root, Hash256, PackedEncoding, TreeHash, TreeHashType};

use crate::{constants::DST, errors::BLSError, PubKey};

#[derive(Debug, PartialEq, Clone)]
pub struct BlsSignature {
    pub inner: [u8; 96],
}

impl Encode for BlsSignature {
    fn is_ssz_fixed_len() -> bool {
        true
    }
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.inner);
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
        let mut signature = [0u8; 96];
        signature.copy_from_slice(bytes);
        Ok(Self { inner: signature })
    }
}

impl Serialize for BlsSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = hex::encode(self.inner);
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
        let mut signature = [0u8; 96];
        signature.copy_from_slice(&result);
        Ok(Self { inner: signature })
    }
}

impl TreeHash for BlsSignature {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        PackedEncoding::from_vec(self.inner.to_vec())
    }

    fn tree_hash_packing_factor() -> usize {
        1
    }

    fn tree_hash_root(&self) -> Hash256 {
        merkle_root(&self.inner, 1)
    }
}

impl BlsSignature {
    pub fn infinity() -> Self {
        Self {
            inner: [
                0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
        }
    }

    fn to_blst_signature(&self) -> Result<BlstSignature, BLSError> {
        BlstSignature::from_bytes(&self.inner).map_err(|e| BLSError::BlstError(e.into()))
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
        let sig = self.to_blst_signature()?;
        let pk = pubkey.to_blst_pubkey()?;

        Ok(sig.verify(true, message, DST, &[], &pk, false) == BLST_ERROR::BLST_SUCCESS)
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
        let sig = self.to_blst_signature()?;
        let public_keys = pubkeys
            .as_ref()
            .iter()
            .map(|key| key.to_blst_pubkey())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(
            sig.fast_aggregate_verify(true, message, DST, &public_keys.iter().collect::<Vec<_>>())
                == BLST_ERROR::BLST_SUCCESS,
        )
    }
}
