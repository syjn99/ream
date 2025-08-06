use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ream_chain_lean::{lean_chain::LeanChain, slot::get_current_slot};
use ream_consensus_misc::constants::lean::INTERVALS_PER_SLOT;
use ream_network_spec::networks::lean_network_spec;
use tokio::{
    sync::RwLock,
    time::{Instant, MissedTickBehavior, interval_at},
};
use tracing::info;

// TODO: We need to replace this after PQC integration.
// For now, we only need ID for keystore.
pub struct LeanKeystore {
    id: u64,
}

/// ValidatorService is responsible for managing validator operations
/// such as proposing blocks and voting on them. This service also holds the keystores
/// for its validators, which are used to sign.
///
/// Every first tick (t=0) it proposes a block if it's the validator's turn.
/// Every second tick (t=1/4) it votes on the proposed block.
///
/// NOTE: Other ticks should be handled by the other services, such as [LeanChainService].
pub struct ValidatorService {
    lean_chain: Arc<RwLock<LeanChain>>,

    keystores: Vec<LeanKeystore>,
}

impl ValidatorService {
    pub async fn new(lean_chain: Arc<RwLock<LeanChain>>, keystores: Vec<LeanKeystore>) -> Self {
        // Hack: If no keystores are provided, create a default one.
        let keystores = if keystores.is_empty() {
            vec![LeanKeystore { id: 0 }] // Placeholder for keystores
        } else {
            keystores
        };

        ValidatorService {
            lean_chain,
            keystores,
        }
    }

    pub async fn start(self) {
        info!("Validator Service started");

        // TODO: Duplicate clock logic from LeanChainService. May need to refactor later.

        // Get the Lean network specification.
        let network_spec = lean_network_spec();
        let seconds_per_slot = network_spec.seconds_per_slot;
        let genesis_time = network_spec.genesis_time;

        // Calculate the genesis instant from the genesis time (in seconds).
        let genesis_instant = UNIX_EPOCH + Duration::from_secs(genesis_time);

        // Assume genesis time is "always" in the future,
        // as we don't support syncing features yet.
        let interval_start = Instant::now()
            + genesis_instant
                .duration_since(SystemTime::now())
                .expect("Genesis time is in the past");

        let mut tick_count = 0u64;
        let mut interval = interval_at(
            interval_start,
            Duration::from_secs(seconds_per_slot / INTERVALS_PER_SLOT),
        );
        interval.set_missed_tick_behavior(MissedTickBehavior::Burst);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match tick_count % 4 {
                        0 => {
                            // First tick (t=0): Propose a block.
                            if let Some(keystore) = self.is_proposer() {
                                info!("Propose block by validator {}", keystore.id);

                                // Acquire the write lock. `accept_new_votes` and `build_block` will modify the lean chain.
                                let mut lean_chain = self.lean_chain.write().await;

                                // Accept new votes and modify the lean chain.
                                lean_chain.accept_new_votes().expect("Failed to accept new votes");

                                // Build a block and propose the block.
                                let new_block = lean_chain.propose_block().expect("Failed to build block");

                                info!(
                                    "Built block for validator {}. Block Info(slot: {}, parent: {}, len(votes): {}, state_root: {})",
                                    keystore.id,
                                    new_block.slot,
                                    new_block.parent,
                                    new_block.votes.len(),
                                    new_block.state_root
                                );

                                // TODO 1: Sign the block with the keystore.
                                // TODO 2: Send the block to the network.
                            } else {
                                info!("Not a proposer, skipping block proposal.");
                            }
                        }
                        1 => {
                            // Second tick (t=1/4): Vote.
                            info!("Vote.");

                            // Build the vote from LeanChain, and modify its validator ID
                            let vote_template = self.lean_chain.read().await.build_vote().expect("Failed to build vote");

                            info!("Built votes for validators: {vote_template:?}");

                            let _votes = self.keystores.iter().map(|ks| {
                                let mut vote = vote_template.clone();
                                vote.validator_id = ks.id;
                                vote
                            }).collect::<Vec<_>>();

                            // TODO 1: Send these votes to `LeanChainService`.
                            // TODO 2: Sign the votes with the keystore.
                            // TODO 3: Send the votes to the network.
                        }
                        _ => {
                            // Other ticks (t=2/4, t=3/4): Do nothing.
                        }
                    }
                    tick_count += 1;
                }
            }
        }
    }

    /// Determine if one of the keystores is the proposer for the current slot.
    fn is_proposer(&self) -> Option<&LeanKeystore> {
        let current_slot = get_current_slot();
        let proposer_index: u64 = current_slot % lean_network_spec().num_validators;

        self.keystores
            .iter()
            .find(|ks| ks.id == proposer_index as u64)
    }
}
