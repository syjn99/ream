use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get};
use alloy_primitives::{Address, aliases::B32};
use ream_api_types_beacon::responses::DataResponse;
use ream_api_types_common::error::ApiError;
use ream_consensus_misc::constants::beacon::{
    DOMAIN_AGGREGATE_AND_PROOF, INACTIVITY_PENALTY_QUOTIENT_BELLATRIX,
};
use ream_network_spec::networks::{BeaconNetworkSpec, beacon_network_spec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct DepositContract {
    #[serde(with = "serde_utils::quoted_u64")]
    chain_id: u64,
    address: Address,
}

impl DepositContract {
    pub fn new(chain_id: u64, address: Address) -> Self {
        Self { chain_id, address }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct SpecConfig {
    deposit_contract_address: Address,
    #[serde(with = "serde_utils::quoted_u64")]
    deposit_network_id: u64,
    domain_aggregate_and_proof: B32,
    #[serde(with = "serde_utils::quoted_u64")]
    inactivity_penalty_quotient: u64,
}

impl From<Arc<BeaconNetworkSpec>> for SpecConfig {
    fn from(network_spec: Arc<BeaconNetworkSpec>) -> Self {
        Self {
            deposit_contract_address: network_spec.deposit_contract_address,
            deposit_network_id: network_spec.deposit_chain_id,
            domain_aggregate_and_proof: DOMAIN_AGGREGATE_AND_PROOF,
            inactivity_penalty_quotient: INACTIVITY_PENALTY_QUOTIENT_BELLATRIX,
        }
    }
}

/// Called by `config/spec` to get specification configuration.
#[get("config/spec")]
pub async fn get_config_spec() -> Result<impl Responder, ApiError> {
    Ok(HttpResponse::Ok().json(DataResponse::new(SpecConfig::from(beacon_network_spec()))))
}

/// Called by `/deposit_contract` to get the Genesis Config of Beacon Chain.
#[get("config/deposit_contract")]
pub async fn get_config_deposit_contract() -> Result<impl Responder, ApiError> {
    let network_spec = beacon_network_spec();
    Ok(
        HttpResponse::Ok().json(DataResponse::new(DepositContract::new(
            network_spec.deposit_chain_id,
            network_spec.deposit_contract_address,
        ))),
    )
}

/// Called by `config/fork_schedule` to get fork schedule
#[get("config/fork_schedule")]
pub async fn get_fork_schedule() -> Result<impl Responder, ApiError> {
    Ok(HttpResponse::Ok().json(DataResponse::new(beacon_network_spec().fork_schedule())))
}
