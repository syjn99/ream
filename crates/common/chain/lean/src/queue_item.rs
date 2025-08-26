use ream_consensus_lean::{block::Block, vote::Vote};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum QueueItem {
    Block(Block),
    Vote(Vote),
}
