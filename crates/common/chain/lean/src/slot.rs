use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ream_network_spec::networks::lean_network_spec;

/// NOTE: Vitalik's implementation of 3SF-mini adds 2 slots more due to the test setup, unlike our
/// implementation.
/// This is due to the fact that his test code starts at slot 1.
pub fn get_current_slot() -> u64 {
    let network_spec = lean_network_spec();
    let seconds_per_slot = network_spec.seconds_per_slot;
    let genesis_time = network_spec.genesis_time;

    let genesis_instant = UNIX_EPOCH + Duration::from_secs(genesis_time);
    let elapsed = genesis_instant
        .duration_since(SystemTime::now())
        .expect("Genesis time is in the past");

    elapsed.as_secs() / seconds_per_slot
}
