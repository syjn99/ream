use std::{
    fmt,
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use alloy_primitives::{B256, hex};
use ethereum_hashing::hash_fixed;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use ssz_derive::{Decode, Encode};
use tree_hash::{PackedEncoding, TreeHash};

use crate::constants::BYTES_PER_COMMITMENT;
pub const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, Hash)]
#[ssz(struct_behaviour = "transparent")]
pub struct KZGCommitment(pub [u8; BYTES_PER_COMMITMENT]);

impl KZGCommitment {
    pub fn calculate_versioned_hash(&self) -> B256 {
        let mut versioned_hash = hash_fixed(&self.0);
        versioned_hash[0] = VERSIONED_HASH_VERSION_KZG;
        B256::from_slice(versioned_hash.as_slice())
    }

    pub fn empty_for_testing() -> Self {
        KZGCommitment([0; BYTES_PER_COMMITMENT])
    }

    pub fn to_fixed_bytes(&self) -> [u8; BYTES_PER_COMMITMENT] {
        self.0
    }
}

impl Display for KZGCommitment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0[0..2] {
            write!(f, "{i:02x}")?;
        }
        write!(f, "…")?;
        for i in &self.0[BYTES_PER_COMMITMENT - 2..BYTES_PER_COMMITMENT] {
            write!(f, "{i:02x}")?;
        }
        Ok(())
    }
}

impl TreeHash for KZGCommitment {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        <[u8; BYTES_PER_COMMITMENT] as TreeHash>::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        self.0.tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        <[u8; BYTES_PER_COMMITMENT] as TreeHash>::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> B256 {
        self.0.tree_hash_root()
    }
}

impl Serialize for KZGCommitment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{self:?}"))
    }
}

impl<'de> Deserialize<'de> for KZGCommitment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl FromStr for KZGCommitment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|e| e.to_string())?;
        if bytes.len() == BYTES_PER_COMMITMENT {
            let mut kzg_commitment_bytes = [0; BYTES_PER_COMMITMENT];
            kzg_commitment_bytes[..].copy_from_slice(&bytes);
            Ok(Self(kzg_commitment_bytes))
        } else {
            Err(format!(
                "InvalidByteLength: got {}, expected {}",
                bytes.len(),
                BYTES_PER_COMMITMENT
            ))
        }
    }
}

impl Debug for KZGCommitment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;

    const COMMITMENT_STR: &str = "0x53fa09af35d1d1a9e76f65e16112a9064ce30d1e4e2df98583f0f5dc2e7dd13a4f421a9c89f518fafd952df76f23adac";

    #[test]
    fn kzg_commitment_display() {
        let display_commitment_str = "0x53fa…adac";
        let display_commitment = KZGCommitment::from_str(COMMITMENT_STR).unwrap().to_string();

        assert_eq!(display_commitment, display_commitment_str);
    }

    #[test]
    fn kzg_commitment_debug() {
        let debug_commitment_str = COMMITMENT_STR;
        let debug_commitment = KZGCommitment::from_str(debug_commitment_str).unwrap();

        assert_eq!(format!("0x{debug_commitment:?}"), debug_commitment_str);
    }

    #[test]
    fn kzg_commitment_tree_hash_root() {
        let commitment = KZGCommitment::from_str(COMMITMENT_STR).unwrap();
        let root = commitment.tree_hash_root();
        let expected_root = commitment.0.tree_hash_root();

        assert_eq!(root, expected_root);
    }
}
