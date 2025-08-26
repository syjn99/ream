use ream_consensus_lean::{block::SignedBlock, vote::SignedVote};

#[derive(Debug, Clone)]
pub enum LeanGossipRequest {
    Block(SignedBlock),
    Vote(SignedVote),
}
