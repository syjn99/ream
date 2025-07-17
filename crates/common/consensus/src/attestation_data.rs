use alloy_primitives::B256;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

use crate::checkpoint::Checkpoint;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct AttestationData {
    // #[serde(with = "serde_utils::quoted_u64")]
    pub slot: u64,
    // #[serde(with = "serde_utils::quoted_u64")]
    pub index: u64,

    /// LMD GHOST vote
    pub beacon_block_root: B256,

    /// FFG vote
    pub source: Checkpoint,
    pub target: Checkpoint,
}
