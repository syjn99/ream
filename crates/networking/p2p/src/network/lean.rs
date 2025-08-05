use std::sync::Arc;

use ream_chain_lean::lean_chain::LeanChain;
use tokio::sync::RwLock;
use tracing::info;

pub struct NetworkService {
    pub time: u64,

    lean_chain: Arc<RwLock<LeanChain>>,
}

impl NetworkService {
    pub async fn new(lean_chain: Arc<RwLock<LeanChain>>) -> Self {
        NetworkService {
            time: 0,

            lean_chain,
        }
    }

    pub async fn start(self) {
        info!("NetworkService started");

        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    }
}
