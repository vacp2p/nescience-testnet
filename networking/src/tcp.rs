use crate::network_protocol::PeerAddr;

#[derive(Debug)]
///Structure, representing peer connection
pub struct Connection {}

#[derive(Debug)]
pub enum ConnectionType {
    Inbound { conn: Connection },
    Outbound { conn: Connection, peer: PeerAddr },
}
