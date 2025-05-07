use alloy_primitives::B256;
use ream_merkle::multiproof::verify_merkle_multiproof;
use ssz_types::{FixedVector, typenum::U8192};

use crate::{constants::EPOCHS_PER_SLASHINGS_VECTOR, misc::compute_epoch_at_slot};

pub trait BeaconStateView {
    fn slot(&self) -> anyhow::Result<u64>;

    fn slashings_mut(&mut self) -> anyhow::Result<&mut FixedVector<u64, U8192>>;
}

pub struct PartialBeaconState {
    pub slot: Option<u64>,
    pub slashings: Option<FixedVector<u64, U8192>>,
}

impl BeaconStateView for PartialBeaconState {
    fn slot(&self) -> anyhow::Result<u64> {
        self.slot
            .ok_or(anyhow::anyhow!("Slot is not set"))
            .map(|slot| slot)
    }

    fn slashings_mut(&mut self) -> anyhow::Result<&mut FixedVector<u64, U8192>> {
        self.slashings
            .as_mut()
            .ok_or(anyhow::anyhow!("Slashings are not set"))
            .map(|slashings| slashings)
    }
}

impl PartialBeaconState {
    pub fn get_current_epoch(&self) -> anyhow::Result<u64> {
        let slot = self.slot.ok_or(anyhow::anyhow!("Slot is not set"))?;
        Ok(compute_epoch_at_slot(slot))
    }

    pub fn process_slashings_reset(&mut self) -> anyhow::Result<()> {
        let next_epoch = self.get_current_epoch()? + 1;

        // Reset slashings
        let slashings = self.slashings_mut()?;
        slashings[(next_epoch % EPOCHS_PER_SLASHINGS_VECTOR) as usize] = 0;

        Ok(())
    }
}

pub struct BeaconStateMultiproof {
    pub leaves: Vec<B256>,
    pub proof: Vec<B256>,
    pub generalized_indices: Vec<u64>,
}

pub struct PartialBeaconStateBuilder {
    pub root: B256,
    pub multiproof: BeaconStateMultiproof,

    pub slot: Option<u64>,
    pub slashings: Option<FixedVector<u64, U8192>>,
}

impl PartialBeaconStateBuilder {
    pub fn from_root(root: B256) -> Self {
        Self {
            root,
            multiproof: BeaconStateMultiproof {
                leaves: vec![],
                proof: vec![],
                generalized_indices: vec![],
            },

            slot: None,
            slashings: None,
        }
    }

    pub fn with_multiproof(
        self,
        leaves: Vec<B256>,
        proof: Vec<B256>,
        generalized_indices: Vec<u64>,
    ) -> Self {
        Self {
            multiproof: BeaconStateMultiproof {
                leaves,
                proof,
                generalized_indices,
            },
            ..self
        }
    }

    pub fn with_slot(self, slot: u64) -> Self {
        Self {
            slot: Some(slot),
            ..self
        }
    }

    pub fn with_slashings(self, slashings: FixedVector<u64, U8192>) -> Self {
        Self {
            slashings: Some(slashings),
            ..self
        }
    }

    pub fn build(self) -> anyhow::Result<PartialBeaconState> {
        let multiproof = self.multiproof;
        verify_merkle_multiproof(
            multiproof.leaves.as_slice(),
            multiproof.proof.as_slice(),
            multiproof.generalized_indices.as_slice(),
            self.root,
        )?;

        Ok(PartialBeaconState {
            slot: self.slot,
            slashings: self.slashings,
        })
    }
}
