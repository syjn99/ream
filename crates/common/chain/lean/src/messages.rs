use ream_consensus_lean::{VoteItem, block::Block};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum LeanChainMessage {
    ProduceBlock {
        slot: u64,
        response: oneshot::Sender<Block>,
    },
    QueueItem(QueueItem),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum QueueItem {
    Block(Block),
    Vote(VoteItem),
}
