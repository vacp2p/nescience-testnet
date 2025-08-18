use serde::{Deserialize, Serialize};

use crate::account::Commitment;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NullifierPublicKey([u8; 32]);

impl From<&NullifierSecretKey> for NullifierPublicKey {
    fn from(_value: &NullifierSecretKey) -> Self {
        todo!()
    }
}

pub type NullifierSecretKey = [u8; 32];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nullifier([u8; 32]);

impl Nullifier {
    pub fn new(commitment: &Commitment, nsk: &NullifierSecretKey) -> Self {
        todo!()
    }
}
