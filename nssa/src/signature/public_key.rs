use std::io::{Cursor, Read};

use serde::{Deserialize, Serialize};

use crate::PrivateKey;

// TODO: Dummy impl. Replace by actual public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(pub(crate) [u8; 32]);

impl PublicKey {
    pub fn new(key: &PrivateKey) -> Self {
        let value = {
            let secret_key = secp256k1::SecretKey::from_byte_array(key.0).unwrap();
            let public_key =
                secp256k1::PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &secret_key);
            let (x_only, _) = public_key.x_only_public_key();
            x_only.serialize()
        };
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PrivateKey, PublicKey};
    fn hex_to_32_bytes(hex: &str) -> [u8; 32] {
        hex::decode(hex).unwrap().try_into().unwrap()
    }

    /// Test vectors from
    /// https://github.com/bitcoin/bips/blob/master/bip-0340/test-vectors.csv
    const BIP340_PK_TEST_VECTORS: [(&str, &str); 5] = [
        (
            "0000000000000000000000000000000000000000000000000000000000000003",
            "F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9",
        ),
        (
            "B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF",
            "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
        ),
        (
            "C90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B14E5C9",
            "DD308AFEC5777E13121FA72B9CC1B7CC0139715309B086C960E18FD969774EB8",
        ),
        (
            "0B432B2677937381AEF05BB02A66ECD012773062CF3FA2549E44F58ED2401710",
            "25D1DFF95105F5253C4022F628A996AD3A0D95FBF21D468A1B33F8C160D8F517",
        ),
        (
            "0340034003400340034003400340034003400340034003400340034003400340",
            "778CAA53B4393AC467774D09497A87224BF9FAB6F6E68B23086497324D6FD117",
        ),
    ];

    #[test]
    fn test_bip340_pk_test_vectors() {
        for (i, (private_key_hex, expected_public_key_hex)) in
            BIP340_PK_TEST_VECTORS.iter().enumerate()
        {
            let key = PrivateKey::try_new(hex_to_32_bytes(private_key_hex)).unwrap();
            let public_key = PublicKey::new(&key);
            let expected_public_key = PublicKey(hex_to_32_bytes(expected_public_key_hex));
            assert_eq!(public_key, expected_public_key, "Failed test vector at index {i}");
        }
    }
}
