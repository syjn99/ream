pub mod constants;
pub mod errors;

#[cfg(feature = "supranational")]
pub mod supranational;
#[cfg(feature = "zkcrypto")]
pub mod zkcrypto;

macro_rules! implement_backend {
    ($backend:path) => {
        pub use $backend::{
            aggregate_pubkey::AggregatePubKey, pubkey::PubKey, signature::BlsSignature,
        };
    };
}

#[cfg(feature = "supranational")]
implement_backend!(supranational);

#[cfg(feature = "zkcrypto")]
implement_backend!(zkcrypto);
