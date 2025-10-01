use ream_consensus_lean::vote::SignedVote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum QueueItem {
    SignedVote(Box<SignedVote>),
}
