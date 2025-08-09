use rs_merkle::Hasher;
use serde::{Deserialize, Serialize};

use crate::{merkle_tree_public::hasher::OwnHasher, transaction::Transaction};
use nssa;

pub type BlockHash = [u8; 32];
pub type Data = Vec<u8>;
pub type BlockId = u64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub block_id: BlockId,
    pub prev_block_id: BlockId,
    pub prev_block_hash: BlockHash,
    pub hash: BlockHash,
    pub transactions: Vec<nssa::PublicTransaction>,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashableBlockData {
    pub block_id: BlockId,
    pub prev_block_id: BlockId,
    pub prev_block_hash: BlockHash,
    pub transactions: Vec<nssa::PublicTransaction>,
    pub data: Data,
}

impl From<HashableBlockData> for Block {
    fn from(value: HashableBlockData) -> Self {
        let data = serde_json::to_vec(&value).unwrap();

        let hash = OwnHasher::hash(&data);

        Self {
            block_id: value.block_id,
            prev_block_id: value.prev_block_id,
            hash,
            transactions: value.transactions,
            data: value.data,
            prev_block_hash: value.prev_block_hash,
        }
    }
}
