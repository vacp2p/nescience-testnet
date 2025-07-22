use std::{collections::HashMap, path::Path};

use anyhow::Result;
use common::{block::Block, merkle_tree_public::TreeHashType, transaction::Transaction};
use storage::RocksDBIO;

pub struct SequecerBlockStore {
    dbio: RocksDBIO,
    tx_hash_to_block_map: HashMap<TreeHashType, u64>,
    pub genesis_id: u64,
}

impl SequecerBlockStore {
    ///Starting database at the start of new chain.
    /// Creates files if necessary.
    ///
    /// ATTENTION: Will overwrite genesis block.
    pub fn open_db_with_genesis(location: &Path, genesis_block: Option<Block>) -> Result<Self> {
        let tx_hash_to_block_map = if let Some(block) = &genesis_block {
            block_to_transactions_map(block)
        } else {
            HashMap::new()
        };

        let dbio = RocksDBIO::new(location, genesis_block)?;

        let genesis_id = dbio.get_meta_first_block_in_db()?;

        Ok(Self {
            dbio,
            genesis_id,
            tx_hash_to_block_map,
        })
    }

    ///Reopening existing database
    pub fn open_db_restart(location: &Path) -> Result<Self> {
        SequecerBlockStore::open_db_with_genesis(location, None)
    }

    pub fn get_block_at_id(&self, id: u64) -> Result<Block> {
        Ok(self.dbio.get_block(id)?)
    }

    pub fn put_block_at_id(&mut self, block: Block) -> Result<()> {
        let new_transactions_map = block_to_transactions_map(&block);
        self.dbio.put_block(block, false)?;
        self.tx_hash_to_block_map.extend(new_transactions_map);
        Ok(())
    }

    pub fn get_transaction_by_hash(&self, hash: TreeHashType) -> Option<Transaction> {
        let block_id = self.tx_hash_to_block_map.get(&hash);
        let block = block_id.map(|&id| self.get_block_at_id(id));
        if let Some(Ok(block)) = block {
            for transaction in block.transactions.into_iter() {
                if transaction.hash() == hash {
                    return Some(transaction);
                }
            }
        }
        None
    }
}

fn block_to_transactions_map(block: &Block) -> HashMap<TreeHashType, u64> {
    block
        .transactions
        .iter()
        .map(|transaction| (transaction.hash(), block.block_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_genesis_block_with_transaction() -> (Block, Transaction) {
        let tx = Transaction {
            tx_kind: common::transaction::TxKind::Public,
            execution_input: Default::default(),
            execution_output: Default::default(),
            utxo_commitments_spent_hashes: Default::default(),
            utxo_commitments_created_hashes: Default::default(),
            nullifier_created_hashes: Default::default(),
            execution_proof_private: Default::default(),
            encoded_data: Default::default(),
            ephemeral_pub_key: Default::default(),
            commitment: Default::default(),
            tweak: Default::default(),
            secret_r: Default::default(),
            sc_addr: Default::default(),
            state_changes: Default::default(),
        };
        (
            Block {
                block_id: 0,
                prev_block_id: 0,
                prev_block_hash: [0; 32],
                hash: [1; 32],
                transactions: vec![tx.clone()],
                data: vec![],
            },
            tx,
        )
    }

    #[test]
    fn test_get_transaction_by_hash_for_existing_transaction() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        let (block, tx) = create_genesis_block_with_transaction();
        let node_store = SequecerBlockStore::open_db_with_genesis(path, Some(block)).unwrap();
        let retrieved_tx = node_store.get_transaction_by_hash(tx.hash());
        assert_eq!(Some(tx), retrieved_tx);
    }

    #[test]
    fn test_get_transaction_by_hash_for_non_existent_transaction() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        let (block, _) = create_genesis_block_with_transaction();
        let node_store = SequecerBlockStore::open_db_with_genesis(path, Some(block)).unwrap();
        let retrieved_tx = node_store.get_transaction_by_hash([0; 32]);
        assert_eq!(None, retrieved_tx);
    }
}
