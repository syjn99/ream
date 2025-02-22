use bls12_381::G1Projective;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

use super::pubkey::PubKey;

#[derive(Debug, PartialEq, Encode, Decode, Clone, Serialize, Deserialize, Default, TreeHash)]
pub struct AggregatePubKey {
    pub inner: PubKey,
}

impl AggregatePubKey {
    pub fn to_pubkey(self) -> PubKey {
        self.inner
    }

    pub fn aggregate(pubkeys: &[&PubKey]) -> anyhow::Result<Self> {
        Ok(Self {
            inner: PubKey {
                inner: pubkeys
                    .iter()
                    .fold(G1Projective::identity(), |acc, pubkey| {
                        acc.add(&pubkey.inner)
                    }),
            },
        })
    }
}
