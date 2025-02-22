use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    pairing, G1Affine, G2Affine, G2Projective,
};

use crate::{
    constants::DST,
    errors::BLSError,
    traits::{Aggregatable, Verifiable, ZkcryptoVerifiable},
    AggregatePubKey, BlsSignature, PubKey,
};

impl TryFrom<BlsSignature> for G2Affine {
    type Error = BLSError;

    fn try_from(value: BlsSignature) -> Result<Self, Self::Error> {
        match G2Affine::from_compressed(
            &value
                .to_bytes()
                .try_into()
                .map_err(|_| BLSError::InvalidByteLength)?,
        )
        .into_option()
        {
            Some(point) => Ok(point),
            None => Err(BLSError::InvalidSignature),
        }
    }
}

impl Verifiable for BlsSignature {
    type Error = BLSError;

    fn verify(&self, pubkey: &PubKey, message: &[u8]) -> Result<bool, BLSError> {
        let h = <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
            [message],
            DST,
        );

        let gt1 = pairing(&G1Affine::try_from(pubkey.clone())?, &G2Affine::from(h));
        let gt2 = pairing(&G1Affine::generator(), &G2Affine::try_from(self.clone())?);

        Ok(gt1 == gt2)
    }

    fn fast_aggregate_verify<'a, P>(&self, pubkeys: P, message: &[u8]) -> Result<bool, BLSError>
    where
        P: AsRef<[&'a PubKey]>,
    {
        let agg_pubkey = AggregatePubKey::aggregate(pubkeys.as_ref())?;

        let h = <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
            [message],
            DST,
        );

        let gt1 = pairing(
            &G1Affine::try_from(agg_pubkey.to_pubkey())?,
            &G2Affine::from(h),
        );
        let gt2 = pairing(&G1Affine::generator(), &G2Affine::try_from(self.clone())?);

        Ok(gt1 == gt2)
    }
}

impl ZkcryptoVerifiable for BlsSignature {}
