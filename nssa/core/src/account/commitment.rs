use serde::{Deserialize, Serialize};

use crate::account::{Account, NullifierPublicKey};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Commitment([u8; 32]);

impl Commitment {
    pub fn new(Npk: &NullifierPublicKey, account: &Account) -> Self {
        todo!()
    }
}
