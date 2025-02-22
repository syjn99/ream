use alloy_primitives::hex;
use bls12_381::{G1Affine, G1Projective};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssz::Encode;
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, FixedVector};
use tree_hash_derive::TreeHash;

use crate::errors::BLSError;

#[derive(Debug, PartialEq, Clone, Encode, Decode, TreeHash, Default)]
pub struct PubKey {
    pub inner: FixedVector<u8, typenum::U48>,
}

impl From<G1Projective> for PubKey {
    fn from(value: G1Projective) -> Self {
        Self {
            inner: G1Affine::from(value).to_compressed().to_vec().into(),
        }
    }
}

impl TryFrom<PubKey> for G1Affine {
    type Error = BLSError;

    fn try_from(value: PubKey) -> Result<Self, Self::Error> {
        match G1Affine::from_compressed(
            &value
                .to_bytes()
                .try_into()
                .map_err(|_| BLSError::InvalidByteLength)?,
        )
        .into_option()
        {
            Some(point) => Ok(point),
            None => Err(BLSError::InvalidPublicKey),
        }
    }
}

impl Serialize for PubKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = hex::encode(self.inner.as_ssz_bytes());
        serializer.serialize_str(&val)
    }
}

impl<'de> Deserialize<'de> for PubKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result: String = Deserialize::deserialize(deserializer)?;
        let result = hex::decode(&result).map_err(serde::de::Error::custom)?;
        let key = FixedVector::from(result);
        Ok(Self { inner: key })
    }
}

impl PubKey {
    pub fn to_bytes(&self) -> &[u8] {
        self.inner.iter().as_slice()
    }
}
