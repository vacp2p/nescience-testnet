use std::collections::HashMap;

use crate::network_protocol::MessageKind;

#[derive(Debug)]
/// Object responsible to manage the rate limits of all network messages
/// for a single connection/peer.
pub struct RateLimiter {
    pub limits: HashMap<MessageKind, u64>,
}

impl RateLimiter {
    pub fn is_allowed(&self, _message: MessageKind) -> bool {
        todo!();
    }
}
