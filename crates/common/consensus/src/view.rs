use std::collections::HashMap;

use alloy_primitives::B256;
use anyhow::ensure;
use ream_merkle::multiproof::verify_merkle_multiproof;
use serde::{Deserialize, Serialize};
use ssz_types::{FixedVector, typenum::U8192};
use tree_hash::TreeHash;

use crate::{constants::EPOCHS_PER_SLASHINGS_VECTOR, misc::compute_epoch_at_slot};

pub const SLOT_GENERALIZED_INDEX: u64 = 66;
pub const SLASHINGS_GENERALIZED_INDEX: u64 = 78;

pub trait BeaconStateView {
    fn slot(&self) -> anyhow::Result<u64>;

    fn slashings(&self) -> anyhow::Result<&FixedVector<u64, U8192>>;
    fn slashings_mut(&mut self) -> anyhow::Result<&mut FixedVector<u64, U8192>>;
}

#[derive(Debug, Clone)]
pub struct PartialBeaconState {
    // BeaconState fields
    pub slot: Option<u64>,
    pub slashings: Option<FixedVector<u64, U8192>>,

    // dirty fields with generalized indices
    pub dirty: Vec<u64>,
}

impl BeaconStateView for PartialBeaconState {
    fn slot(&self) -> anyhow::Result<u64> {
        self.slot.ok_or(anyhow::anyhow!("Slot is not set"))
    }

    fn slashings(&self) -> anyhow::Result<&FixedVector<u64, U8192>> {
        self.slashings
            .as_ref()
            .ok_or(anyhow::anyhow!("Slashings are not set"))
    }

    fn slashings_mut(&mut self) -> anyhow::Result<&mut FixedVector<u64, U8192>> {
        self.slashings
            .as_mut()
            .ok_or(anyhow::anyhow!("Slashings are not set"))
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

        // Mark dirty
        self.dirty.push(SLASHINGS_GENERALIZED_INDEX);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconStateMultiproof {
    pub leaves: Vec<B256>,
    pub proof: Vec<B256>,
    pub generalized_indices: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn with_slashings(self, slashings: &FixedVector<u64, U8192>) -> Self {
        Self {
            slashings: Some(slashings.clone()),
            ..self
        }
    }

    pub fn build(self) -> anyhow::Result<PartialBeaconState> {
        let multiproof = self.multiproof;

        let generalized_index_to_leave: HashMap<u64, B256> = multiproof
            .generalized_indices
            .iter()
            .zip(multiproof.leaves.iter())
            .map(|(index, leaf)| (*index, *leaf))
            .collect();

        if self.slot.is_some() {
            let slot = self.slot.expect("Slot is not set");
            ensure!(
                slot.to_le_bytes().tree_hash_root()
                    == *generalized_index_to_leave
                        .get(&SLOT_GENERALIZED_INDEX)
                        .expect("Slot not found in multiproof"),
                "Slot does not match multiproof"
            );
        }

        if self.slashings.is_some() {
            let slashings_root = self
                .slashings
                .as_ref()
                .expect("Slashings are not set")
                .tree_hash_root();
            ensure!(
                slashings_root
                    == *generalized_index_to_leave
                        .get(&SLASHINGS_GENERALIZED_INDEX)
                        .expect("Slashings not found in multiproof"),
                "Slashings do not match multiproof"
            );
        }

        verify_merkle_multiproof(
            multiproof.leaves.as_slice(),
            multiproof.proof.as_slice(),
            multiproof.generalized_indices.as_slice(),
            self.root,
        )?;

        Ok(PartialBeaconState {
            slot: self.slot,
            slashings: self.slashings,

            dirty: vec![],
        })
    }
}
