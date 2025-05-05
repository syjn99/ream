use alloy_primitives::Address;
use ream_bls::{BLSSignature, PubKey};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct SignedBLSToExecutionChange {
    pub message: BLSToExecutionChange,
    pub signature: BLSSignature,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct BLSToExecutionChange {
    pub validator_index: u64,
    pub from_bls_pubkey: PubKey,
    pub to_execution_address: Address,
}
