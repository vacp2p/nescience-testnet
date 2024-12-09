use std::path::Path;

use anyhow::{Result, anyhow};
use storage::{block::Block, RocksDBIO};

pub struct NodeBlockStore {
    dbio: RocksDBIO,
}

impl NodeBlockStore {
    ///Starting database at the start of new chain.
    /// Creates files if necessary.
    ///
    /// ATTENTION: Will overwrite genesis block.
    pub fn open_db_with_genesis(location: &Path, genesis_block: Option<Block>) -> Result<Self> {
        Ok(Self {
            dbio: RocksDBIO::new(location, genesis_block)?,
        })
    }

    ///Reopening existing database
    pub fn open_db_restart(location: &Path) -> Result<Self> {
        NodeBlockStore::db_destroy(location)?;
        NodeBlockStore::open_db_with_genesis(location, None)
    }

    ///Destroying existing database
    fn db_destroy(location: &Path) -> Result<()> {
        RocksDBIO::destroy(location).map_err(|err| anyhow!("RocksDBIO error: {}", err))
    }

    pub fn get_block_at_id(&self, id: u64) -> Result<Block> {
        Ok(self.dbio.get_block(id)?)
    }

    pub fn put_block_at_id(&self, block: Block) -> Result<()> {
        Ok(self.dbio.put_block(block, false)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use storage::block::{Block, BlockHash, BlockId, Data};
    use storage::transaction::Transaction;

    fn create_genesis_block() -> Block {
        Block {
            block_id: 0,
            prev_block_id: 0,
            prev_block_hash: [0; 32],
            hash: [1; 32],
            transactions: vec![],
            data: Data::default(),
        }
    }

    fn create_sample_block(block_id: u64, prev_block_id: u64) -> Block {
        Block {
            block_id: block_id,
            prev_block_id: prev_block_id,
            prev_block_hash: [0; 32],
            hash: [1; 32],
            transactions: vec![],
            data: Data::default(),
        }
    }

    #[test]
    fn test_open_db_with_genesis() {
        let temp_dir: tempfile::TempDir = tempdir().unwrap();
        let path = temp_dir.path();

        let genesis_block = create_genesis_block();
        let node_store = NodeBlockStore::open_db_with_genesis(path, Some(genesis_block.clone()))
            .unwrap();

        // Verify the genesis block is stored
        let stored_block = node_store.get_block_at_id(0).unwrap();
        assert_eq!(stored_block.block_id, genesis_block.block_id);
        assert_eq!(stored_block.hash, genesis_block.hash);
    }

}
