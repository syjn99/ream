#![no_main]
sp1_zkvm::entrypoint!(main);

use ream_consensus::{
    deneb::{beacon_block::SignedBeaconBlock, beacon_state::BeaconState},
    execution_engine::mock_engine::MockExecutionEngine,
};
use ream_state_executor::BeaconStateExecutor;
use ssz::Encode;

pub fn main() {
    let state = sp1_zkvm::io::read::<BeaconState>();
    let block = sp1_zkvm::io::read::<SignedBeaconBlock>();

    let mut executor = BeaconStateExecutor::new(state, MockExecutionEngine::new());
    executor
        .process_new_block_sync(&block)
        .expect("failed to process block");

    let post_state_bytes = executor.beacon_state.as_ssz_bytes();

    sp1_zkvm::io::commit_slice(&post_state_bytes);
}
