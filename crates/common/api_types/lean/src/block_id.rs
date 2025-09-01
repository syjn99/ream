use std::str::FromStr;

use alloy_primitives::{B256, hex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockID {
    Finalized,
    Genesis,
    Head,
    Justified,
    Slot(u64),
    /// expected to be a 0x-prefixed hex string.
    Root(B256),
}

impl Serialize for BlockID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BlockID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "finalized" => Ok(BlockID::Finalized),
            "genesis" => Ok(BlockID::Genesis),
            "head" => Ok(BlockID::Head),
            "justified" => Ok(BlockID::Justified),
            _ => {
                if s.starts_with("0x") {
                    B256::from_str(&s)
                        .map(BlockID::Root)
                        .map_err(|_| serde::de::Error::custom(format!("Invalid hex root: {s}")))
                } else if s.chars().all(|c| c.is_ascii_digit()) {
                    s.parse::<u64>()
                        .map(BlockID::Slot)
                        .map_err(|_| serde::de::Error::custom(format!("Invalid slot number: {s}")))
                } else {
                    Err(serde::de::Error::custom(format!("Invalid state ID: {s}")))
                }
            }
        }
    }
}

impl std::fmt::Display for BlockID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockID::Finalized => write!(f, "finalized"),
            BlockID::Genesis => write!(f, "genesis"),
            BlockID::Head => write!(f, "head"),
            BlockID::Justified => write!(f, "justified"),
            BlockID::Slot(slot) => write!(f, "{slot}"),
            BlockID::Root(root) => write!(f, "0x{}", hex::encode(root)),
        }
    }
}
