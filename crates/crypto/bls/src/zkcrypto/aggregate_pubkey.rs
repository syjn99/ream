use bls12_381::{G1Affine, G1Projective};

use crate::{
    errors::BLSError,
    traits::{Aggregatable, ZkcryptoAggregatable},
    AggregatePubKey, PubKey,
};

impl Aggregatable for AggregatePubKey {
    type Error = BLSError;

    fn aggregate(pubkeys: &[&PubKey]) -> Result<Self, Self::Error> {
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

impl ZkcryptoAggregatable for AggregatePubKey {}
