use anyhow::Result;

use crate::{network_protocol::PeerId, peer::Peer};

#[derive(Debug)]
///Entrypoint to network module.
/// Manages connections with peers in network
pub struct PeerManager {
    pub my_peer_id: PeerId,
}

impl PeerManager {
    pub async fn start_peer_manager(_num_threads: u8, my_peer_id: PeerId) -> Result<Self> {
        Ok(Self { my_peer_id })
    }

    pub async fn connect(&self, _peer_id: PeerId) -> Peer {
        todo!()
    }
}
