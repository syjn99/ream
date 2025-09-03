use std::collections::HashMap;

use alloy_primitives::B256;
use anyhow::anyhow;
use ream_consensus_lean::{
    block::{Block, BlockBody},
    checkpoint::Checkpoint,
    get_fork_choice_head, get_latest_justified_hash, is_justifiable_slot, process_block,
    state::LeanState,
    vote::Vote,
};
use ream_metrics::{PROPOSE_BLOCK_TIME, start_timer_vec, stop_timer};
use ream_network_spec::networks::lean_network_spec;
use ream_sync::rwlock::{Reader, Writer};
use tree_hash::TreeHash;

use crate::slot::get_current_slot;

pub type LeanChainWriter = Writer<LeanChain>;
pub type LeanChainReader = Reader<LeanChain>;

/// [LeanChain] represents the state that the Lean node should maintain.
///
/// Most of the fields are based on the Python implementation of [`Staker`](https://github.com/ethereum/research/blob/d225a6775a9b184b5c1fd6c830cc58a375d9535f/3sf-mini/p2p.py#L15-L42),
/// but doesn't include `validator_id` as a node should manage multiple validators.
#[derive(Clone, Debug, Default)]
pub struct LeanChain {
    pub chain: HashMap<B256, Block>,
    pub post_states: HashMap<B256, LeanState>,
    pub known_votes: Vec<Vote>,
    pub new_votes: Vec<Vote>,
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
            .map(|state| state.latest_finalized.root)
    }

    /// Compute the latest block that the staker is allowed to choose as the target
    pub fn compute_safe_target(&self) -> anyhow::Result<B256> {
        let justified_hash = get_latest_justified_hash(&self.post_states)
            .ok_or_else(|| anyhow!("No justified hash found in post states"))?;

        get_fork_choice_head(
            &self.chain,
            &justified_hash,
            &self.new_votes,
            self.num_validators * 2 / 3,
        )
    }

    /// Process new votes that the staker has received. Vote processing is done
    /// at a particular time, because of safe target and view merge rule
    pub fn accept_new_votes(&mut self) -> anyhow::Result<()> {
        for new_vote in self.new_votes.drain(..) {
            if !self.known_votes.contains(&new_vote) {
                self.known_votes.push(new_vote);
            }
        }

        self.recompute_head()?;
        Ok(())
    }

    /// Done upon processing new votes or a new block
    pub fn recompute_head(&mut self) -> anyhow::Result<()> {
        let justified_hash = get_latest_justified_hash(&self.post_states)
            .ok_or_else(|| anyhow!("Failed to get latest_justified_hash from post_states"))?;
        self.head = get_fork_choice_head(&self.chain, &justified_hash, &self.known_votes, 0)?;
        Ok(())
    }

    pub fn propose_block(&self, slot: u64) -> anyhow::Result<Block> {
        let initialize_block_timer = start_timer_vec(&PROPOSE_BLOCK_TIME, &["initialize_block"]);
        let head_state = self
            .post_states
            .get(&self.head)
            .ok_or_else(|| anyhow!("Post state not found for head: {}", self.head))?;
        let mut new_block = Block {
            slot,
            proposer_index: slot % lean_network_spec().num_validators,
            parent_root: self.head,
            // Diverged from Python implementation: Using `B256::ZERO` instead of `None`)
            state_root: B256::ZERO,
            body: BlockBody::default(),
        };
        stop_timer(initialize_block_timer);

        let mut state: LeanState;

        // Keep attempt to add valid votes from the list of available votes
        let add_votes_timer = start_timer_vec(&PROPOSE_BLOCK_TIME, &["add_valid_votes_to_block"]);
        loop {
            state = process_block(head_state, &new_block)?;

            let new_votes_to_add = self
                .known_votes
                .clone()
                .into_iter()
                .filter(|vote| vote.source.root == state.latest_justified.root)
                .filter(|vote| !new_block.body.votes.contains(vote))
                .collect::<Vec<_>>();

            if new_votes_to_add.is_empty() {
                break;
            }

            for vote in new_votes_to_add {
                new_block
                    .body
                    .votes
                    .push(vote)
                    .map_err(|err| anyhow!("Failed to add vote to new_block: {err:?}"))?;
            }
        }
        stop_timer(add_votes_timer);

        // Compute the state root
        let compute_state_root_timer =
            start_timer_vec(&PROPOSE_BLOCK_TIME, &["compute_state_root"]);
        new_block.state_root = state.tree_hash_root();
        stop_timer(compute_state_root_timer);

        Ok(new_block)
    }

    pub fn build_vote(&self) -> anyhow::Result<Vote> {
        let state = self
            .post_states
            .get(&self.head)
            .ok_or_else(|| anyhow!("Post state not found for head: {}", self.head))?;
        let mut target_block = self
            .chain
            .get(&self.head)
            .ok_or_else(|| anyhow!("Block not found in chain for head: {}", self.head))?;

        // If there is no very recent safe target, then vote for the k'th ancestor
        // of the head
        for _ in 0..3 {
            let safe_target_block = self.chain.get(&self.safe_target).ok_or_else(|| {
                anyhow!("Block not found for safe target hash: {}", self.safe_target)
            })?;
            if target_block.slot > safe_target_block.slot {
                target_block = self.chain.get(&target_block.parent_root).ok_or_else(|| {
                    anyhow!(
                        "Block not found for target block's parent hash: {}",
                        target_block.parent_root
                    )
                })?;
            }
        }

        // If the latest finalized slot is very far back, then only some slots are
        // valid to justify, make sure the target is one of those
        while !is_justifiable_slot(&state.latest_finalized.slot, &target_block.slot) {
            target_block = self.chain.get(&target_block.parent_root).ok_or_else(|| {
                anyhow!(
                    "Block not found for target block's parent hash: {}",
                    target_block.parent_root
                )
            })?;
        }

        let head_block = self
            .chain
            .get(&self.head)
            .ok_or_else(|| anyhow!("Block not found for head: {}", self.head))?;

        Ok(Vote {
            // NOTE: This is a placeholder for `validator_id`.
            // This field will eventually be set by the `ValidatorService` with the actual validator
            // IDs.
            validator_id: 0,
            slot: get_current_slot(),
            head: Checkpoint {
                root: self.head,
                slot: head_block.slot,
            },
            target: Checkpoint {
                root: target_block.tree_hash_root(),
                slot: target_block.slot,
            },
            source: Checkpoint {
                root: state.latest_justified.root,
                slot: state.latest_justified.slot,
            },
        })
    }

    pub fn get_block_by_root(&self, root: B256) -> Option<Block> {
        self.chain.get(&root).cloned()
    }

    pub fn get_block_by_slot(&self, slot: u64) -> Option<Block> {
        self.chain
            .values()
            .find(|block| block.slot == slot)
            .cloned()
    }
}
