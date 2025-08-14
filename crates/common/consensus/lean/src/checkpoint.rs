use alloy_primitives::B256;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

#[derive(
    Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash,
)]
pub struct Checkpoint {
    pub root: B256,
    pub slot: u64,
}
