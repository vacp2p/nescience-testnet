use crate::{
    network_protocol::{HandshakeFailedReason, PeerAddr},
    tcp::Connection,
};

#[derive(Debug)]
/// Structure, which stores all of the peer interaction data.
/// Created at per-peer connection basis at `PeerManager`
pub struct Peer {
    pub connection: Connection,
    pub peer_addr: PeerAddr,
}

impl Peer {
    pub fn handshake(&mut self) -> Result<(), HandshakeFailedReason> {
        todo!();
    }
}
