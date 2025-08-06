use std::sync::Arc;

use ream_chain_lean::lean_chain::LeanChain;
use tokio::sync::RwLock;
use tracing::info;

/// NetworkService is responsible for the following:
/// 1. Peer discovery and management.
/// 2. Gossiping blocks and votes.
///
/// TBD: It will be best if we reuse the existing NetworkManagerService for the beacon node.
pub struct NetworkService {
    lean_chain: Arc<RwLock<LeanChain>>,
}

impl NetworkService {
    pub async fn new(lean_chain: Arc<RwLock<LeanChain>>) -> Self {
        NetworkService { lean_chain }
    }

    pub async fn start(self) {
        info!("NetworkService started");
        info!(
            "Current LeanChain head: {}",
            self.lean_chain.read().await.head
        );
    }
}
