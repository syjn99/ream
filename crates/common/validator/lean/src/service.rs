use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ream_network_spec::networks::lean_network_spec;
use tokio::time::{Instant, MissedTickBehavior, interval_at};
use tracing::info;

/// 3SF-mini divides a slot into 4 intervals.
/// Reference: https://github.com/ethereum/research/blob/d225a6775a9b184b5c1fd6c830cc58a375d9535f/3sf-mini/p2p.py#L77-L98
const INTERVALS_PER_SLOT: u64 = 4;

/// ValidatorService is responsible for managing validator operations
/// such as proposing blocks and voting on them.
///
/// Every first tick (t=0) it proposes a block if it's the validator's turn.
/// Every second tick (t=1/4) it votes on the proposed block.
///
/// NOTE: Other ticks should be handled by the other services, such as the consensus service.
pub struct ValidatorService {}

impl ValidatorService {
    pub async fn new() -> Self {
        ValidatorService {}
    }

    pub async fn start(self) {
        info!("Validator Service started");

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
