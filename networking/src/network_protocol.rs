#[derive(Debug)]
pub enum MessageKind {}

pub type PeerId = u64;
pub type PeerDistance = u32;

#[derive(Debug)]
pub struct PeerAddr {
    pub id: PeerId,
    //Probably will be socket address in the future
    pub addr: String,
}

#[derive(Debug)]
///Structure, which contains all necessary fields for handshake
pub struct Handshake {}

#[derive(Debug)]
pub enum HandshakeFailedReason {}
