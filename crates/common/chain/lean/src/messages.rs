use ream_consensus_lean::{
    block::{Block, SignedBlock},
    vote::SignedVote,
};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum LeanChainServiceMessage {
    ProduceBlock {
        slot: u64,
        response: oneshot::Sender<Block>,
    },
    ProcessBlock {
        signed_block: SignedBlock,
        is_trusted: bool,
        need_gossip: bool,
    },
    ProcessVote {
        signed_vote: SignedVote,
        is_trusted: bool,
        need_gossip: bool,
    },
}
