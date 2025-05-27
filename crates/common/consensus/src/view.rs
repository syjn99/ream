use alloy_primitives::{B256, aliases::B32};
use anyhow::ensure;
use ethereum_hashing::{hash, hash_fixed};
use ream_merkle::multiproof::Multiproof;
use serde::{Deserialize, Serialize};
use ssz_types::{
    FixedVector, VariableList,
    typenum::{U8192, U65536, U1099511627776, Unsigned},
};
use tree_hash::TreeHash;

use crate::{
    beacon_block_header::BeaconBlockHeader,
    constants::{
        BEACON_STATE_LATEST_BLOCK_HEADER_GENERALIZED_INDEX,
        BEACON_STATE_RANDAO_MIXES_GENERALIZED_INDEX, BEACON_STATE_SLASHINGS_GENERALIZED_INDEX,
        BEACON_STATE_SLOT_GENERALIZED_INDEX, BEACON_STATE_VALIDATORS_GENERALIZED_INDEX,
        DOMAIN_BEACON_PROPOSER, EPOCHS_PER_HISTORICAL_VECTOR, EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_EFFECTIVE_BALANCE_ELECTRA, MAX_RANDOM_VALUE, MIN_SEED_LOOKAHEAD,
    },
    electra::beacon_block::BeaconBlock,
    misc::{bytes_to_int64, compute_epoch_at_slot, compute_shuffled_index},
    validator::Validator,
};

pub trait CoreView {
    fn slot(&self) -> anyhow::Result<u64>;
}

pub trait LatestBlockHeaderView: CoreView {
    fn latest_block_header(&self) -> anyhow::Result<&BeaconBlockHeader>;
    fn latest_block_header_mut(&mut self) -> anyhow::Result<&mut BeaconBlockHeader>;
}

pub trait ValidatorView: CoreView {
    fn validators(&self) -> anyhow::Result<&VariableList<Validator, U1099511627776>>;
    fn validators_mut(&mut self) -> anyhow::Result<&mut VariableList<Validator, U1099511627776>>;
}

pub trait SlashingsView: CoreView {
    fn slashings(&self) -> anyhow::Result<&FixedVector<u64, U8192>>;
    fn slashings_mut(&mut self) -> anyhow::Result<&mut FixedVector<u64, U8192>>;
}

pub trait RandaoMixesView: CoreView {
    fn randao_mixes(&self) -> anyhow::Result<&FixedVector<B256, U65536>>;
    fn randao_mixes_mut(&mut self) -> anyhow::Result<&mut FixedVector<B256, U65536>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialBeaconState {
    // BeaconState fields
    pub slot: Option<u64>,

    pub latest_block_header: Option<BeaconBlockHeader>,

    pub validators: Option<VariableList<Validator, U1099511627776>>,

    pub randao_mixes: Option<FixedVector<B256, U65536>>,
    pub slashings: Option<FixedVector<u64, U8192>>,

    // dirty fields with generalized indices
    pub dirty: Vec<u64>,
}

impl CoreView for PartialBeaconState {
    fn slot(&self) -> anyhow::Result<u64> {
        self.slot.ok_or(anyhow::anyhow!("Slot is not set"))
    }
}

impl LatestBlockHeaderView for PartialBeaconState {
    fn latest_block_header(&self) -> anyhow::Result<&BeaconBlockHeader> {
        self.latest_block_header
            .as_ref()
            .ok_or(anyhow::anyhow!("Latest block header is not set"))
    }

    fn latest_block_header_mut(&mut self) -> anyhow::Result<&mut BeaconBlockHeader> {
        self.latest_block_header
            .as_mut()
            .ok_or(anyhow::anyhow!("Latest block header is not set"))
    }
}

impl ValidatorView for PartialBeaconState {
    fn validators(&self) -> anyhow::Result<&VariableList<Validator, U1099511627776>> {
        self.validators
            .as_ref()
            .ok_or(anyhow::anyhow!("Validators are not set"))
    }

    fn validators_mut(&mut self) -> anyhow::Result<&mut VariableList<Validator, U1099511627776>> {
        self.validators
            .as_mut()
            .ok_or(anyhow::anyhow!("Validators are not set"))
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

impl RandaoMixesView for PartialBeaconState {
    fn randao_mixes(&self) -> anyhow::Result<&FixedVector<B256, U65536>> {
        self.randao_mixes
            .as_ref()
            .ok_or(anyhow::anyhow!("Randao mixes are not set"))
    }

    fn randao_mixes_mut(&mut self) -> anyhow::Result<&mut FixedVector<B256, U65536>> {
        self.randao_mixes
            .as_mut()
            .ok_or(anyhow::anyhow!("Randao mixes are not set"))
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

    /// Return the randao mix at a recent ``epoch``.
    pub fn get_randao_mix(&self, epoch: u64) -> anyhow::Result<B256> {
        Ok(self.randao_mixes()?[(epoch % EPOCHS_PER_HISTORICAL_VECTOR) as usize])
    }

    /// Return the sequence of active validator indices at ``epoch``.
    pub fn get_active_validator_indices(&self, epoch: u64) -> anyhow::Result<Vec<u64>> {
        Ok(self
            .validators()?
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if v.is_active_validator(epoch) {
                    Some(i as u64)
                } else {
                    None
                }
            })
            .collect())
    }

    /// Return the seed at ``epoch``.
    pub fn get_seed(&self, epoch: u64, domain_type: B32) -> anyhow::Result<B256> {
        let mix =
            self.get_randao_mix(epoch + EPOCHS_PER_HISTORICAL_VECTOR - MIN_SEED_LOOKAHEAD - 1)?;
        let epoch_with_index =
            [domain_type.as_slice(), &epoch.to_le_bytes(), mix.as_slice()].concat();
        Ok(B256::from(hash_fixed(&epoch_with_index)))
    }

    /// Return from ``indices`` a random index sampled by effective balance
    pub fn compute_proposer_index(&self, indices: &[u64], seed: B256) -> anyhow::Result<u64> {
        ensure!(!indices.is_empty(), "Index must be less than index_count");

        let mut i: usize = 0;
        let total = indices.len();

        loop {
            let candidate_index = indices[compute_shuffled_index(i % total, total, seed)?];

            let random_bytes = hash(&[seed.as_slice(), &(i / 16).to_le_bytes()].concat());
            let offset = i % 16 * 2;
            let random_value = bytes_to_int64(&random_bytes[offset..offset + 2]);

            let effective_balance = self.validators()?[candidate_index as usize].effective_balance;

            if (effective_balance * MAX_RANDOM_VALUE)
                >= (MAX_EFFECTIVE_BALANCE_ELECTRA * random_value as u64)
            {
                return Ok(candidate_index);
            }

            i += 1;
        }
    }

    /// Return the beacon proposer index at the current slot.
    pub fn get_beacon_proposer_index(&self) -> anyhow::Result<u64> {
        let epoch = self.get_current_epoch()?;
        let seed = B256::from(hash_fixed(
            &[
                self.get_seed(epoch, DOMAIN_BEACON_PROPOSER)?.as_slice(),
                &self.slot()?.to_le_bytes(),
            ]
            .concat(),
        ));
        let indices = self.get_active_validator_indices(epoch)?;
        self.compute_proposer_index(&indices, seed)
    }

    pub fn process_block_header(&mut self, block: &BeaconBlock) -> anyhow::Result<()> {
        // Verify that the slots match
        ensure!(
            self.slot()? == block.slot,
            "State slot must be equal to block slot"
        );
        // Verify that the block is newer than latest block header
        ensure!(
            block.slot > self.latest_block_header()?.slot,
            "Block slot must be greater than latest block header slot of state"
        );
        // Verify that proposer index is the correct index
        ensure!(
            block.proposer_index == self.get_beacon_proposer_index()?,
            "Block proposer index must be equal to beacon proposer index"
        );
        // Verify that the parent matches
        ensure!(
            block.parent_root == self.latest_block_header()?.tree_hash_root(),
            "Block Parent Root must be equal root of latest block header"
        );

        // Cache current block as the new latest block
        *self.latest_block_header_mut()? = BeaconBlockHeader {
            slot: block.slot,
            proposer_index: block.proposer_index,
            parent_root: block.parent_root,
            state_root: B256::default(), // Overwritten in the next process_slot call
            body_root: block.body.tree_hash_root(),
        };

        // Verify proposer is not slashed
        let proposer = &self.validators()?[block.proposer_index as usize];
        ensure!(!proposer.slashed, "Block proposer must not be slashed");

        // Mark dirty
        self.dirty
            .push(BEACON_STATE_LATEST_BLOCK_HEADER_GENERALIZED_INDEX);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialBeaconStateBuilder {
    pub root: B256,
    pub multiproof: Multiproof,

    pub slot: Option<u64>,
    pub latest_block_header: Option<BeaconBlockHeader>,
    pub validators: Option<Vec<Validator>>,
    pub randao_mixes: Option<FixedVector<B256, U65536>>,
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

    pub fn with_latest_block_header(self, latest_block_header: &BeaconBlockHeader) -> Self {
        Self {
            latest_block_header: Some(latest_block_header.clone()),
            ..self
        }
    }

    pub fn with_validators<T>(self, validators: T) -> Self
    where
        T: IntoIterator<Item = Validator>,
    {
        Self {
            validators: Some(validators.into_iter().collect()),
            ..self
        }
    }

    pub fn with_randao_mixes(self, randao_mixes: &FixedVector<B256, U65536>) -> Self {
        Self {
            randao_mixes: Some(randao_mixes.clone()),
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

        if self.latest_block_header.is_some() {
            let latest_block_header_root = self
                .latest_block_header
                .as_ref()
                .expect("Latest block header is not set")
                .tree_hash_root();
            ensure!(
                latest_block_header_root
                    == multiproof
                        .leaves
                        .get(&BEACON_STATE_LATEST_BLOCK_HEADER_GENERALIZED_INDEX)
                        .expect("Index not found")
                        .tree_hash_root(),
                "Latest block header does not match multiproof"
            );
        }

        let validators = if let Some(validators) = self.validators {
            let validator_variable_list: VariableList<Validator, U1099511627776> =
                VariableList::from(validators);

            ensure!(
                validator_variable_list.len() <= U1099511627776::to_usize(),
                "Validators list exceeds maximum length"
            );

            ensure!(
                validator_variable_list.len() > 0,
                "Validators list must not be empty"
            );

            let expected_root = multiproof
                .leaves
                .get(&BEACON_STATE_VALIDATORS_GENERALIZED_INDEX)
                .expect("Index not found")
                .tree_hash_root();

            println!("expected root: {:?}", expected_root);
            println!("N::to_usize(): {}", U1099511627776::to_usize());
            println!(
                "next_power_of_two: {}",
                U1099511627776::to_usize().next_power_of_two()
            );
            println!(
                "depth: {}",
                get_depth(U1099511627776::to_usize().next_power_of_two()) + 1
            );
            println!(
                "validator_variable_list root: {:?}",
                validator_variable_list.tree_hash_root()
            );

            ensure!(
                validator_variable_list.tree_hash_root()
                    == multiproof
                        .leaves
                        .get(&BEACON_STATE_VALIDATORS_GENERALIZED_INDEX)
                        .expect("Index not found")
                        .tree_hash_root(),
                "Validators do not match multiproof"
            );
            Some(VariableList::from(validator_variable_list))
        } else {
            None
        };

        if self.randao_mixes.is_some() {
            let randao_mixes_root = self
                .randao_mixes
                .as_ref()
                .expect("Randao mixes are not set")
                .tree_hash_root();
            ensure!(
                randao_mixes_root
                    == multiproof
                        .leaves
                        .get(&BEACON_STATE_RANDAO_MIXES_GENERALIZED_INDEX)
                        .expect("Index not found")
                        .tree_hash_root(),
                "Randao mixes do not match multiproof"
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
            latest_block_header: self.latest_block_header,
            validators,
            randao_mixes: self.randao_mixes,
            slashings: self.slashings,

            dirty: vec![],
        })
    }
}

fn get_depth(i: usize) -> usize {
    let total_bits = std::mem::size_of::<usize>() * 8;
    total_bits - i.leading_zeros() as usize - 1
}
