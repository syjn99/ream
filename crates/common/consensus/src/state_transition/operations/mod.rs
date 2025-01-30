pub mod errors;

use std::result::Result;

use crate::{
    attestation::Attestation,
    attester_slashing::AttesterSlashing,
    bls_to_execution_change::SignedBLSToExecutionChange,
    deneb::{
        beacon_block::BeaconBlock, beacon_block_body::BeaconBlockBody, beacon_state::BeaconState,
        execution_payload::ExecutionPayload,
    },
    deposit::Deposit,
    proposer_slashing::ProposerSlashing,
    sync_aggregate::SyncAggregate,
    voluntary_exit::SignedVoluntaryExit,
};

impl BeaconState {
    pub fn process_attestation(
        &mut self,
        _attestation: &Attestation,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_attestation not yet implemented");
    }

    pub fn process_attester_slashing(
        &mut self,
        _attester_slashing: &AttesterSlashing,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_attester_slashing not yet implemented");
    }

    pub fn process_block_header(
        &mut self,
        _beacon_block: &BeaconBlock,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_block_header not yet implemented");
    }

    pub fn process_deposit(
        &mut self,
        _deposit: &Deposit,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_deposit not yet implemented");
    }

    pub fn process_proposer_slashing(
        &mut self,
        _proposer_slashing: &ProposerSlashing,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_proposer_slashing not yet implemented");
    }

    pub fn process_voluntary_exit(
        &mut self,
        _voluntary_exit: &SignedVoluntaryExit,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_voluntary_exit not yet implemented");
    }

    pub fn process_sync_aggregate(
        &mut self,
        _sync_aggregate: &SyncAggregate,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_sync_aggregate not yet implemented");
    }

    pub fn process_execution_payload(
        &mut self,
        _beacon_block_body: &BeaconBlockBody,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_execution_payload not yet implemented");
    }

    pub fn process_withdrawals(
        &mut self,
        _execution_payload: &ExecutionPayload,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_withdrawals not yet implemented");
    }

    pub fn process_bls_to_execution_change(
        &mut self,
        _bls_to_execution_change: &SignedBLSToExecutionChange,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_bls_to_execution_change not yet implemented");
    }
}
