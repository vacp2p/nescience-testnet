use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{PrivateKey, PublicKey, error::NssaError, public_transaction::Message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    value: [u8; 64],
}

impl Signature {
    pub(crate) fn new(key: &PrivateKey, message: &[u8]) -> Self {
        let value = {
            let secp = secp256k1::Secp256k1::new();
            let secret_key = secp256k1::SecretKey::from_byte_array(key.0).unwrap();
            let keypair = secp256k1::Keypair::from_secret_key(&secp, &secret_key);
            let signature = secp.sign_schnorr_no_aux_rand(message, &keypair);
            signature.to_byte_array()
        };
        Self { value }
    }

    pub fn is_valid_for(&self, bytes: &[u8], public_key: &PublicKey) -> bool {
        let pk = secp256k1::XOnlyPublicKey::from_byte_array(public_key.0).unwrap();
        let secp = secp256k1::Secp256k1::new();
        let sig = secp256k1::schnorr::Signature::from_byte_array(self.value);
        secp.verify_schnorr(&sig, &bytes, &pk).is_ok()
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a slice
        serializer.serialize_bytes(&self.value)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = Deserialize::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::invalid_length(
                bytes.len(),
                &"expected 64 bytes",
            ));
        }
        let mut value = [0u8; 64];
        value.copy_from_slice(bytes);
        Ok(Signature { value })
    }
}
