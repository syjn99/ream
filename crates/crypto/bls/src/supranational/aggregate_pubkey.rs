use blst::min_pk::AggregatePublicKey as BlstAggregatePublicKey;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

use crate::PubKey;

#[derive(Debug, PartialEq, Clone, Encode, Decode, TreeHash, Serialize, Deserialize, Default)]
pub struct AggregatePubKey {
    pub inner: PubKey,
}

impl AggregatePubKey {
    pub fn to_pubkey(self) -> PubKey {
        self.inner
    }

    pub fn aggregate(pubkeys: &[&PubKey]) -> anyhow::Result<Self> {
        let blst_pubkeys = pubkeys
            .iter()
            .map(|pk| pk.to_blst_pubkey())
            .collect::<Result<Vec<_>, _>>()?;
        let aggregate_pubkey =
            BlstAggregatePublicKey::aggregate(&blst_pubkeys.iter().collect::<Vec<_>>(), true)
                .map_err(|err| {
                    anyhow::anyhow!("Failed to aggregate and validate public keys {err:?}")
                })?;
        Ok(Self {
            inner: aggregate_pubkey.to_public_key().into(),
        })
    }
}
