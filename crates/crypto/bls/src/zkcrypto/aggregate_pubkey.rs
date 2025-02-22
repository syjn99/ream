use bls12_381::{G1Affine, G1Projective};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

use super::pubkey::PubKey;
use crate::errors::BLSError;

#[derive(Debug, PartialEq, Encode, Decode, Clone, Serialize, Deserialize, Default, TreeHash)]
pub struct AggregatePubKey {
    pub inner: PubKey,
}

impl AggregatePubKey {
    pub fn to_pubkey(self) -> PubKey {
        self.inner
    }

    pub fn aggregate(pubkeys: &[&PubKey]) -> Result<Self, BLSError> {
        let agg_point = pubkeys
            .iter()
            .try_fold(G1Projective::identity(), |acc, pubkey| {
                let point: G1Affine = (*pubkey).clone().try_into()?;
                Ok(acc.add(&G1Projective::from(point)))
            })?;

        Ok(Self {
            inner: PubKey::from(agg_point),
        })
    }
}
