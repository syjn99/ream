use alloy_primitives::B256;
use anyhow::ensure;
use ream_merkle::multiproof::Multiproof;
use serde::{Deserialize, Serialize};
use ssz_types::{FixedVector, typenum::U8192};
use tree_hash::TreeHash;

use crate::{
    constants::{
        BEACON_STATE_SLASHINGS_GENERALIZED_INDEX, BEACON_STATE_SLOT_GENERALIZED_INDEX,
        EPOCHS_PER_SLASHINGS_VECTOR,
    },
    misc::compute_epoch_at_slot,
};

pub trait CoreView {
    fn slot(&self) -> anyhow::Result<u64>;
}

pub trait SlashingsView: CoreView {
    fn slashings(&self) -> anyhow::Result<&FixedVector<u64, U8192>>;
    fn slashings_mut(&mut self) -> anyhow::Result<&mut FixedVector<u64, U8192>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialBeaconState {
    // BeaconState fields
    pub slot: Option<u64>,
    pub slashings: Option<FixedVector<u64, U8192>>,

    // dirty fields with generalized indices
    pub dirty: Vec<u64>,
}

impl CoreView for PartialBeaconState {
    fn slot(&self) -> anyhow::Result<u64> {
        self.slot.ok_or(anyhow::anyhow!("Slot is not set"))
    }
}

impl SlashingsView for PartialBeaconState {
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
        self.dirty.push(BEACON_STATE_SLASHINGS_GENERALIZED_INDEX);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialBeaconStateBuilder {
    pub root: B256,
    pub multiproof: Multiproof,

    pub slot: Option<u64>,
    pub slashings: Option<FixedVector<u64, U8192>>,
}

impl PartialBeaconStateBuilder {
    pub fn from_root(root: B256) -> Self {
        Self {
            root,
            ..Self::default()
        }
    }

    pub fn with_multiproof(self, multiproof: Multiproof) -> Self {
        Self { multiproof, ..self }
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

        if self.slot.is_some() {
            let slot = self.slot.expect("Slot is not set");
            ensure!(
                slot.to_le_bytes().tree_hash_root()
                    == multiproof
                        .leaves
                        .get(&BEACON_STATE_SLOT_GENERALIZED_INDEX)
                        .expect("Index not found")
                        .tree_hash_root(),
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
                    == multiproof
                        .leaves
                        .get(&BEACON_STATE_SLASHINGS_GENERALIZED_INDEX)
                        .expect("Index not found")
                        .tree_hash_root(),
                "Slashings do not match multiproof"
            );
        }

        multiproof.verify(self.root)?;

        Ok(PartialBeaconState {
            slot: self.slot,
            slashings: self.slashings,

            dirty: vec![],
        })
    }
}
