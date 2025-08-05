use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ream_consensus_misc::constants::lean::INTERVALS_PER_SLOT;
use ream_network_spec::networks::lean_network_spec;
use tokio::{
    sync::RwLock,
    time::{Instant, MissedTickBehavior, interval_at},
};
use tracing::{debug, info};

use crate::lean_chain::LeanChain;

pub struct LeanChainService {
    lean_chain: Arc<RwLock<LeanChain>>,
}

impl LeanChainService {
    pub async fn new(lean_chain: Arc<RwLock<LeanChain>>) -> Self {
        LeanChainService { lean_chain }
    }

    pub async fn start(self) {
        info!("Lean Chain Service started");

        // TODO: Duplicate clock logic from ValidatorService. May need to refactor later.

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
                        2 => {
                            // Third tick (t=2/4): Compute the safe target.
                            debug!("Compute safe target.");
                            let mut lean_chain = self.lean_chain.write().await;
                            lean_chain.safe_target = lean_chain.compute_safe_target().expect("Failed to compute safe target");
                        }
                        3 => {
                            // Fourth tick (t=3/4): Accept new votes.
                            debug!("Accept new votes.");
                            self.lean_chain.write().await.accept_new_votes().expect("Failed to accept new votes");
                        }
                        _ => {
                            // Other ticks (t=0, t=1/4): Do nothing.
                        }
                    }
                    tick_count += 1;
                }
            }
        }
    }
}
