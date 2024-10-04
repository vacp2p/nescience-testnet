use std::sync::Arc;

use networking::peer_manager::PeerManager;
use tokio::sync::Mutex;

#[derive(Debug)]
///Entrypoint to consensus.
/// Manages consensus protocol.
pub struct ConsensusManager {
    pub peer_manager: Arc<Mutex<PeerManager>>,
}

impl ConsensusManager {
    pub fn new(peer_manager: Arc<Mutex<PeerManager>>) -> Self {
        Self { peer_manager }
    }

    //ToDo: change block from generic value into struct, when data block will be defined
    pub fn vote(&self, _block: serde_json::Value) -> bool {
        todo!()
    }
}
