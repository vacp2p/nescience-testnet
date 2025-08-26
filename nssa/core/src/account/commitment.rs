use risc0_zkvm::{
    serde::to_vec,
    sha::{Impl, Sha256},
};
use serde::{Deserialize, Serialize};

use crate::account::{Account, NullifierPublicKey};

#[derive(Serialize, Deserialize)]
#[cfg_attr(any(feature = "host", test), derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct Commitment(pub(super) [u8; 32]);

impl Commitment {
    pub fn new(Npk: &NullifierPublicKey, account: &Account) -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&Npk.to_byte_array());
        let account_bytes_with_hashed_data = {
            let mut this = Vec::new();
            for word in &account.program_owner {
                this.extend_from_slice(&word.to_le_bytes());
            }
            this.extend_from_slice(&account.balance.to_le_bytes());
            this.extend_from_slice(&account.nonce.to_le_bytes());
            let hashed_data: [u8; 32] = Impl::hash_bytes(&account.data)
                .as_bytes()
                .try_into()
                .unwrap();
            this.extend_from_slice(&hashed_data);
            this
        };
        bytes.extend_from_slice(&account_bytes_with_hashed_data);
        Self(Impl::hash_bytes(&bytes).as_bytes().try_into().unwrap())
    }
}
