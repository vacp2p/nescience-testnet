use crate::{PrivateKey, error::NssaError};

// TODO: Dummy impl. Replace by actual public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    pub fn new_from_private_key(key: &PrivateKey) -> Self {
        let value = {
            let secret_key = secp256k1::SecretKey::from_byte_array(*key.value()).unwrap();
            let public_key =
                secp256k1::PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &secret_key);
            let (x_only, _) = public_key.x_only_public_key();
            x_only.serialize()
        };
        Self(value)
    }

    pub fn new(value: [u8; 32]) -> Result<Self, NssaError> {
        // Check point is valid
        let _ = secp256k1::XOnlyPublicKey::from_byte_array(value)
            .map_err(|_| NssaError::InvalidPublicKey)?;
        Ok(Self(value))
    }

    pub fn value(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::{PublicKey, signature::bip340_test_vectors};

    #[test]
    fn test_public_key_generation_from_bip340_test_vectors() {
        for (i, test_vector) in bip340_test_vectors::test_vectors().into_iter().enumerate() {
            let Some(private_key) = &test_vector.seckey else {
                continue;
            };
            let public_key = PublicKey::new_from_private_key(private_key);
            let expected_public_key = &test_vector.pubkey;
            assert_eq!(
                &public_key, expected_public_key,
                "Failed test vector at index {i}"
            );
        }
    }
}
