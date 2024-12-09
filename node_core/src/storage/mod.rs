use std::path::Path;

use accounts::account_core::{Account, AccountAddress};
use accounts_store::NodeAccountsStore;
use anyhow::Result;
use block_store::NodeBlockStore;
use storage::{
    block::Block,
    merkle_tree_public::merkle_tree::{PublicTransactionMerkleTree, UTXOCommitmentsMerkleTree},
    nullifier::UTXONullifier,
    nullifier_sparse_merkle_tree::NullifierSparseMerkleTree,
    utxo_commitment::UTXOCommitment,
};

pub mod accounts_store;
pub mod block_store;

pub struct NodeChainStore {
    pub acc_store: NodeAccountsStore,
    pub block_store: NodeBlockStore,
    pub nullifier_store: NullifierSparseMerkleTree,
    pub utxo_commitments_store: UTXOCommitmentsMerkleTree,
    pub pub_tx_store: PublicTransactionMerkleTree,
    ///For simplicity, we will allow only one account per node.
    /// ToDo: Change it in future
    node_main_account_info: Account,
}

impl NodeChainStore {
    pub fn new_with_genesis(home_dir: &Path, genesis_block: Block) -> Self {
        let acc_store = NodeAccountsStore::default();
        let nullifier_store = NullifierSparseMerkleTree::default();
        let utxo_commitments_store = UTXOCommitmentsMerkleTree::new(vec![]);
        let pub_tx_store = PublicTransactionMerkleTree::new(vec![]);

        //Sequencer should panic if unable to open db,
        //as fixing this issue may require actions non-native to program scope
        let block_store =
            NodeBlockStore::open_db_with_genesis(&home_dir.join("rocksdb"), Some(genesis_block))
                .unwrap();

        Self {
            acc_store,
            block_store,
            nullifier_store,
            utxo_commitments_store,
            pub_tx_store,
            node_main_account_info: Account::new(),
        }
    }

    pub fn get_main_account_addr(&self) -> AccountAddress {
        self.node_main_account_info.address
    }

    pub fn dissect_insert_block(&mut self, block: Block) -> Result<()> {
        for tx in &block.transactions {
            self.utxo_commitments_store.add_tx_multiple(
                tx.utxo_commitments_created_hashes
                    .clone()
                    .into_iter()
                    .map(|hash| UTXOCommitment { hash })
                    .collect(),
            );

            self.nullifier_store.insert_items(
                tx.nullifier_created_hashes
                    .clone()
                    .into_iter()
                    .map(|hash| UTXONullifier { utxo_hash: hash })
                    .collect(),
            )?;

            self.pub_tx_store.add_tx(tx.clone());
        }

        self.block_store.put_block_at_id(block)?;

        Ok(())
    }
}
