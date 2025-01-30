pub mod errors;

use std::result::Result;

use crate::{attestation::Attestation, deneb::beacon_state::BeaconState};

impl BeaconState {
    pub fn process_attestation(
        &mut self,
        attestation: &Attestation,
    ) -> Result<(), errors::BlockOperationError> {
        unimplemented!("process_attestation not yet implemented");
    }
}
