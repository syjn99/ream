use ream_consensus::{
    deneb::{beacon_block::SignedBeaconBlock, beacon_state::BeaconState},
    execution_engine::engine_trait::ExecutionApi,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconStateExecutor<E: ExecutionApi> {
    /// The canonical consensus state (a BeaconState).
    pub beacon_state: BeaconState,

    /// An Execution Engine client or trait object used for verifying or updating execution payloads.
    pub execution_api: E,

    /// Blocks that should be processed in the next round.
    pub pending_blocks: Vec<SignedBeaconBlock>,
}

impl<E: ExecutionApi> BeaconStateExecutor<E> {
    /// Create a new Executor, associating a `BeaconState`, and
    /// an implementation of the `ExecutionApi`.
    pub fn new(beacon_state: BeaconState, execution_api: E) -> Self {
        Self {
            beacon_state,
            execution_api,
            pending_blocks: Vec::new(),
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

    /// Process pending blocks in the executor.
    pub async fn process_pending_blocks(&mut self) -> anyhow::Result<()> {
        let pending_blocks = std::mem::take(&mut self.pending_blocks);

        for block in pending_blocks {
            self.process_new_block(&block).await?;
        }
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
