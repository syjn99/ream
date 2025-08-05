use std::collections::HashMap;

use alloy_primitives::B256;
use ream_consensus_lean::{
    QueueItem, block::Block, get_fork_choice_head, get_latest_justified_hash, state::LeanState,
    vote::Vote,
};
use tree_hash::TreeHash;

#[derive(Clone, Debug)]
pub struct LeanChain {
    pub chain: HashMap<B256, Block>,
    pub post_states: HashMap<B256, LeanState>,
    pub known_votes: Vec<Vote>,
    pub new_votes: Vec<Vote>,
    pub dependencies: HashMap<B256, Vec<QueueItem>>,
    pub genesis_hash: B256,
    pub num_validators: u64,
    pub safe_target: B256,
    pub head: B256,
}

impl LeanChain {
    pub fn new(genesis_block: Block, genesis_state: LeanState) -> LeanChain {
        let genesis_hash = genesis_block.tree_hash_root();

        LeanChain {
            // Votes that we have received and taken into account
            known_votes: Vec::new(),
            // Votes that we have received but not yet taken into account
            new_votes: Vec::new(),
            // Objects that we will process once we have processed their parents
            dependencies: HashMap::new(),
            // Initialize the chain with the genesis block
            genesis_hash,
            num_validators: genesis_state.config.num_validators,
            // Block that it is safe to use to vote as the target
            // Diverge from Python implementation: Use genesis hash instead of `None`
            safe_target: genesis_hash,
            // Head of the chain
            head: genesis_hash,
            // {block_hash: block} for all blocks that we know about
            chain: HashMap::from([(genesis_hash, genesis_block)]),
            // {block_hash: post_state} for all blocks that we know about
            post_states: HashMap::from([(genesis_hash, genesis_state)]),
        }
    }

    pub fn latest_justified_hash(&self) -> Option<B256> {
        get_latest_justified_hash(&self.post_states)
    }

    pub fn latest_finalized_hash(&self) -> Option<B256> {
        self.post_states
            .get(&self.head)
            .map(|state| state.latest_finalized_hash)
    }

    /// Compute the latest block that the staker is allowed to choose as the target
    fn compute_safe_target(&self) -> anyhow::Result<B256> {
        let justified_hash = get_latest_justified_hash(&self.post_states)
            .ok_or_else(|| anyhow::anyhow!("No justified hash found in post states"))?;

        get_fork_choice_head(
            &self.chain,
            &justified_hash,
            &self.new_votes,
            self.num_validators * 2 / 3,
        )
    }

    /// Process new votes that the staker has received. Vote processing is done
    /// at a particular time, because of safe target and view merge rule
    fn accept_new_votes(&mut self) -> anyhow::Result<()> {
        for new_vote in self.new_votes.drain(..) {
            if !self.known_votes.contains(&new_vote) {
                self.known_votes.push(new_vote);
            }
        }

        self.recompute_head()?;
        Ok(())
    }

    /// Done upon processing new votes or a new block
    fn recompute_head(&mut self) -> anyhow::Result<()> {
        let justified_hash = get_latest_justified_hash(&self.post_states).ok_or_else(|| {
            anyhow::anyhow!("Failed to get latest_justified_hash from post_states")
        })?;
        self.head = get_fork_choice_head(&self.chain, &justified_hash, &self.known_votes, 0)?;
        Ok(())
    }

    // TODO: Add necessary methods for processs_block, vote, and receive.
}
