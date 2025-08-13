use serde::{Deserialize, Serialize};

use crate::error::NssaError;

// TODO: Dummy impl. Replace by actual private key.
// TODO: Remove Debug, Clone, Serialize, Deserialize, PartialEq and Eq for security reasons
// TODO: Implement Zeroize
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivateKey([u8; 32]);

impl PrivateKey {
    fn is_valid_key(value: [u8; 32]) -> bool {
        secp256k1::SecretKey::from_byte_array(value).is_ok()
    }

    pub fn try_new(value: [u8; 32]) -> Result<Self, NssaError> {
        if Self::is_valid_key(value) {
            Ok(Self(value))
        } else {
            Err(NssaError::InvalidPrivateKey)
        }
    }

    pub fn value(&self) -> &[u8; 32] {
        &self.0
    }
}
