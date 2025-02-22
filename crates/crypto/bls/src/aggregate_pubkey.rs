use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

use crate::pubkey::PubKey;

#[derive(Debug, PartialEq, Clone, Encode, Decode, TreeHash, Serialize, Deserialize, Default)]
pub struct AggregatePubKey {
    pub inner: PubKey,
}

impl AggregatePubKey {
    pub fn to_pubkey(self) -> PubKey {
        self.inner
    }
}
