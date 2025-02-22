use alloy_primitives::hex;
use bls12_381::{G1Affine, G1Projective};
use serde::{de::Error as SerdeError, Deserialize, Deserializer, Serialize, Serializer};
use ssz::{Decode, Encode};
use tree_hash::{merkle_root, Hash256, PackedEncoding, TreeHash, TreeHashType};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PubKey {
    pub inner: G1Projective,
}

impl PubKey {
    fn validate_point(point: &G1Affine) -> Result<(), String> {
        if bool::from(point.is_identity()) {
            return Err("Invalid point: infinity of G1".to_string());
        }

        if !bool::from(point.is_torsion_free()) {
            return Err("Invalid torsion component".to_string());
        }

        Ok(())
    }
}

impl Encode for PubKey {
    fn is_ssz_fixed_len() -> bool {
        true
    }
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&G1Affine::from(&self.inner).to_compressed());
    }
    fn ssz_bytes_len(&self) -> usize {
        48
    }
    fn ssz_fixed_len() -> usize {
        48
    }
}

impl Decode for PubKey {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        48
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let point = match G1Affine::from_compressed(bytes.try_into().map_err(|_| {
            ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 48,
            }
        })?)
        .into_option()
        {
            Some(p) => p,
            None => {
                return Err(ssz::DecodeError::BytesInvalid(
                    "Invalid public key".to_string(),
                ))
            }
        };

        Self::validate_point(&point).map_err(|e| ssz::DecodeError::BytesInvalid(e.to_string()))?;

        Ok(Self {
            inner: G1Projective::from(point),
        })
    }
}

impl Serialize for PubKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&G1Affine::from(&self.inner).to_compressed())
    }
}

impl<'de> Deserialize<'de> for PubKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result: String = Deserialize::deserialize(deserializer)?;
        let result = hex::decode(&result).map_err(SerdeError::custom)?;
        let mut pubkey = [0u8; 48];
        pubkey.copy_from_slice(&result);

        let point = match G1Affine::from_compressed(&pubkey).into_option() {
            Some(p) => p,
            None => return Err(SerdeError::custom("Invalid public key")),
        };

        Self::validate_point(&point).map_err(SerdeError::custom)?;

        Ok(Self {
            inner: G1Projective::from(point),
        })
    }
}

impl TreeHash for PubKey {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        PackedEncoding::from_vec(G1Affine::from(&self.inner).to_compressed().to_vec())
    }

    fn tree_hash_packing_factor() -> usize {
        1
    }

    fn tree_hash_root(&self) -> Hash256 {
        merkle_root(&G1Affine::from(&self.inner).to_compressed(), 1)
    }
}

impl PubKey {
    pub fn to_bytes(&self) -> [u8; 48] {
        G1Affine::from(&self.inner).to_compressed()
    }
}
