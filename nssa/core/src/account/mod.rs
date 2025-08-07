mod commitment;
mod nullifier;

pub(crate) use commitment::Commitment;
pub(crate) use nullifier::Nullifier;
use serde::{Deserialize, Serialize};

use crate::program::ProgramId;

pub type Nonce = u128;
type Data = Vec<u8>;

/// Account to be used both in public and private contexts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub program_owner: ProgramId,
    pub balance: u128,
    pub data: Data,
    pub nonce: Nonce,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountWithMetadata {
    pub account: Account,
    pub is_authorized: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            program_owner: [0; 8],
            balance: 0,
            data: vec![],
            nonce: 0,
        }
    }
}
