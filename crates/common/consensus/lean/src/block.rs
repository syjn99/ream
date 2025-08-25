use alloy_primitives::B256;
use ream_post_quantum_crypto::PQSignature;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{VariableList, typenum::U4096};
use tree_hash_derive::TreeHash;

use crate::vote::Vote;

/// Represents a signed block in the Lean chain.
///
/// See the [Lean specification](https://github.com/leanEthereum/leanSpec/blob/main/docs/client/containers.md#signedblock)
/// for detailed protocol information.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct SignedBlock {
    pub message: Block,
    pub signature: PQSignature,
}

/// Represents a block in the Lean chain.
///
/// See the [Lean specification](https://github.com/leanEthereum/leanSpec/blob/main/docs/client/containers.md#block)
/// for detailed protocol information.
#[derive(
    Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash,
)]
pub struct Block {
    pub slot: u64,
    pub proposer_index: u64,
    // Diverged from Python implementation: Disallow `None` (uses `B256::ZERO` instead)
    pub parent_root: B256,
    // Diverged from Python implementation: Disallow `None` (uses `B256::ZERO` instead)
    pub state_root: B256,
    pub body: BlockBody,
}

/// Represents the body of a block in the Lean chain.
///
/// See the [Lean specification](https://github.com/leanEthereum/leanSpec/blob/main/docs/client/containers.md#blockbody)
/// for detailed protocol information.
#[derive(
    Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash,
)]
pub struct BlockBody {
    pub votes: VariableList<Vote, U4096>,
}
