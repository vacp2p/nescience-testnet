use serde::{Deserialize, Serialize};

use crate::signature::PublicKey;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    value: [u8; 32],
}

impl Address {
    pub fn new(value: [u8; 32]) -> Self {
        Self { value }
    }

    pub fn from_public_key(public_key: &PublicKey) -> Self {
        // TODO: Check specs
        Address::new(*public_key.value())
    }

    pub fn value(&self) -> &[u8; 32] {
        &self.value
    }
}
