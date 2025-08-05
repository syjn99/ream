use alloy_primitives::B256;
use ream_consensus_lean::{block::Block, state::LeanState};
use ream_network_spec::networks::lean_network_spec;
use ssz_types::VariableList;
use tree_hash::TreeHash;

fn genesis_block(state_root: B256) -> Block {
    Block {
        slot: 1,
        parent: B256::ZERO,
        votes: VariableList::empty(),
        state_root,
    }
}

fn genesis_state(num_validators: u64) -> LeanState {
    LeanState::new(num_validators)
}

/// Setup the genesis block and state for the Lean chain.
///
/// Reference: https://github.com/ethereum/research/blob/d225a6775a9b184b5c1fd6c830cc58a375d9535f/3sf-mini/test_p2p.py#L119-L131
pub fn setup_genesis() -> (Block, LeanState) {
    let genesis_state = genesis_state(lean_network_spec().num_validators);
    let genesis_block = genesis_block(genesis_state.tree_hash_root());

    (genesis_block, genesis_state)
}
