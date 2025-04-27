use ream_consensus::{
    deneb::{beacon_block::SignedBeaconBlock, beacon_state::BeaconState},
    execution_engine::engine_trait::ExecutionApi,
};
use ream_fork_choice::store::Store;
use tokio::sync::mpsc;
use tree_hash::TreeHash;

#[derive(Debug)]
pub struct BeaconStateExecutor<E: ExecutionApi> {
    /// The in-memory fork-choice store tracking blocks, checkpoints, etc.
    pub store: Store,

    /// The canonical consensus state (a BeaconState).
    pub beacon_state: BeaconState,

    /// An Execution Engine client or trait object used for verifying or updating execution payloads.
    pub execution_api: E,
}

impl<E: ExecutionApi> BeaconStateExecutor<E> {
    /// Create a new Executor, associating a `Store``, a `BeaconState`, and
    /// an implementation of the `ExecutionApi`.
    pub fn new(store: Store, beacon_state: BeaconState, execution_api: E) -> Self {
        Self {
            store,
            beacon_state,
            execution_api,
        }
    }

    /// Process a newly arrived signed block (from gossip or other means).
    pub async fn process_new_block(
        &mut self,
        signed_block: &SignedBeaconBlock,
    ) -> anyhow::Result<()> {
        // Run the state transition logic (slot processing, block processing, signature checks, etc.)
        let validate_signature = true;
        self.beacon_state
            .state_transition(signed_block, validate_signature, &self.execution_api)
            .await?;

        Ok(())
    }

    fn on_transition_end(&mut self, signed_block: &SignedBeaconBlock) -> anyhow::Result<()> {
        let block_root = signed_block.message.tree_hash_root();

        // Store the new block and its state in the fork-choice store
        self.store
            .blocks
            .insert(block_root, signed_block.message.clone());
        self.store
            .block_states
            .insert(block_root, self.beacon_state.clone());

        // TODO: Store checkpoints, if needed

        Ok(())
    }

    /// A minimal example of a routine that waits for new blocks from a channel (e.g., a network
    /// service) and processes them in a loop.
    pub async fn start_executor_loop(
        &mut self,
        mut incoming_blocks: mpsc::Receiver<SignedBeaconBlock>,
    ) -> anyhow::Result<()> {
        while let Some(block) = incoming_blocks.recv().await {
            match self.process_new_block(&block).await {
                Ok(()) => {
                    self.on_transition_end(&block)?;
                    println!("Processed block at slot {}", block.message.slot);
                }
                Err(e) => {
                    eprintln!("Error processing block: {:?}", e);
                }
            }
        }
        Ok(())
    }
}
