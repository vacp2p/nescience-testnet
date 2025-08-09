use common::transaction::SignaturePublicKey;
use tiny_keccak::{Hasher, Keccak};

// TODO: Consider wrapping `AccountAddress` in a struct.

pub type AccountAddress = [u8; 32];
