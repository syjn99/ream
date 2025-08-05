use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ream_chain_lean::lean_chain::LeanChain;
use ream_consensus_misc::constants::lean::INTERVALS_PER_SLOT;
use ream_network_spec::networks::lean_network_spec;
use tokio::{
    sync::RwLock,
    time::{Instant, MissedTickBehavior, interval_at},
};
use tracing::info;

/// ValidatorService is responsible for managing validator operations
/// such as proposing blocks and voting on them.
///
/// Every first tick (t=0) it proposes a block if it's the validator's turn.
/// Every second tick (t=1/4) it votes on the proposed block.
///
/// NOTE: Other ticks should be handled by the other services, such as the consensus service.
pub struct ValidatorService {
    lean_chain: Arc<RwLock<LeanChain>>,
}

impl ValidatorService {
    pub async fn new(lean_chain: Arc<RwLock<LeanChain>>) -> Self {
        ValidatorService { lean_chain }
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
                            info!("Propose block if it's my turn.");
                        }
                        1 => {
                            // Second tick (t=1/4): Vote.
                            info!("Vote.");
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
}
