use serde::{Deserialize, Serialize};

use crate::public_transaction::Message;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Signature(pub(crate) u8);

// TODO: Dummy impl. Replace by actual private key.
// TODO: Remove Debug, Clone, Serialize, Deserialize, PartialEq and Eq for security reasons
// TODO: Implement Zeroize
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivateKey(pub(crate) u8);

impl PrivateKey {
    pub fn new(dummy_value: u8) -> Self {
        Self(dummy_value)
    }
}

// TODO: Dummy impl. Replace by actual public key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicKey(pub(crate) u8);

impl PublicKey {
    pub fn new(key: &PrivateKey) -> Self {
        // TODO: implement
        Self(key.0)
    }
}

impl Signature {
    pub(crate) fn new(key: &PrivateKey, message: &[u8]) -> Self {
        Signature(key.0)
    }

    pub fn is_valid_for(&self, _message: &Message, public_key: &PublicKey) -> bool {
        // TODO: implement
        self.0 == public_key.0
    }
}
