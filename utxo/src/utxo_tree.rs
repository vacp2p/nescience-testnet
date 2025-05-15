use std::collections::{BTreeMap, HashMap};

use common::merkle_tree_public::TreeHashType;
use monotree::database::MemoryDB;
use monotree::hasher::Blake3;
use monotree::{Hasher, Monotree, Proof};

use crate::utxo_core::UTXO;

#[derive(Debug, Clone)]
pub struct UTXOTreeInput {
    pub utxo_id: u64,
    pub tx_id: u64,
    pub block_id: u64,
    pub utxo: UTXO,
}

#[derive(Debug, Clone)]
pub struct TreeTxWithUTXOId {
    pub id: u64,
    pub utxos: BTreeMap<u64, UTXO>,
}

#[derive(Debug, Clone)]
pub struct TreeBlockWithTxId {
    pub id: u64,
    pub txs: BTreeMap<u64, TreeTxWithUTXOId>,
}

pub struct UTXOSparseMerkleTree {
    pub curr_root: Option<TreeHashType>,
    pub tree: Monotree<MemoryDB, Blake3>,
    pub hasher: Blake3,
    pub store: HashMap<TreeHashType, UTXO>,
    pub leafs: BTreeMap<u64, TreeBlockWithTxId>,
}

impl UTXOSparseMerkleTree {
    pub fn new() -> Self {
        UTXOSparseMerkleTree {
            curr_root: None,
            tree: Monotree::default(),
            hasher: Blake3::new(),
            store: HashMap::new(),
            leafs: BTreeMap::new(),
        }
    }

    pub fn modify_leavs_with_nullifier_input(&mut self, tree_utxo: UTXOTreeInput) {
        self.leafs
            .entry(tree_utxo.block_id)
            .and_modify(|tree_block| {
                tree_block
                    .txs
                    .entry(tree_utxo.tx_id)
                    .and_modify(|tree_tx| {
                        tree_tx.utxos.insert(tree_utxo.utxo_id, tree_utxo.utxo);
                    })
                    .or_insert(TreeTxWithUTXOId {
                        id: tree_utxo.tx_id,
                        utxos: BTreeMap::new(),
                    });
            })
            .or_insert(TreeBlockWithTxId {
                id: tree_utxo.block_id,
                txs: BTreeMap::new(),
            });
    }

    pub fn insert_item(&mut self, tree_utxo: UTXOTreeInput) -> Result<(), monotree::Errors> {
        let root = self.curr_root.as_ref();

        let new_root = self
            .tree
            .insert(root, &tree_utxo.utxo.hash, &tree_utxo.utxo.hash)?;

        self.curr_root = new_root;

        self.store
            .insert(tree_utxo.utxo.hash, tree_utxo.utxo.clone());
        self.modify_leavs_with_nullifier_input(tree_utxo);

        Ok(())
    }

    pub fn insert_items(&mut self, tree_utxos: Vec<UTXOTreeInput>) -> Result<(), monotree::Errors> {
        let root = self.curr_root.as_ref();

        let hashes: Vec<TreeHashType> = tree_utxos.iter().map(|item| item.utxo.hash).collect();

        let new_root = self.tree.inserts(root, &hashes, &hashes)?;

        for tree_utxo in tree_utxos {
            self.store
                .insert(tree_utxo.utxo.hash, tree_utxo.utxo.clone());
            self.modify_leavs_with_nullifier_input(tree_utxo);
        }

        self.curr_root = new_root;

        Ok(())
    }

    pub fn get_item(&mut self, hash: TreeHashType) -> Result<Option<&UTXO>, monotree::Errors> {
        let hash = self.tree.get(self.curr_root.as_ref(), &hash)?;

        Ok(hash.and_then(|hash| self.store.get(&hash)))
    }

    pub fn get_membership_proof(
        &mut self,
        nullifier_hash: TreeHashType,
    ) -> Result<Option<Proof>, monotree::Errors> {
        self.tree
            .get_merkle_proof(self.curr_root.as_ref(), &nullifier_hash)
    }
}

impl Default for UTXOSparseMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use common::AccountId;

    use super::*;
    use crate::utxo_core::{UTXOPayload, UTXO};

    fn sample_utxo_payload(amount: u128) -> UTXOPayload {
        UTXOPayload {
            owner: AccountId::default(),
            asset: vec![1, 2, 3],
            amount,
            privacy_flag: false,
        }
    }

    fn sample_utxo(amount: u128) -> anyhow::Result<UTXO> {
        UTXO::create_utxo_from_payload(sample_utxo_payload(amount))
    }

    fn sample_utxo_input(
        utxo_id: u64,
        tx_id: u64,
        block_id: u64,
        amount: u128,
    ) -> anyhow::Result<UTXOTreeInput> {
        sample_utxo(amount).map(|utxo| UTXOTreeInput {
            utxo_id,
            tx_id,
            block_id,
            utxo,
        })
    }

    #[test]
    fn test_utxo_sparse_merkle_tree_new() {
        let smt = UTXOSparseMerkleTree::new();
        assert!(smt.curr_root.is_none());
        assert_eq!(smt.store.len(), 0);
    }

    #[test]
    fn test_insert_item() {
        let mut smt = UTXOSparseMerkleTree::new();
        let utxo = sample_utxo_input(1, 1, 1, 10).unwrap();

        let result = smt.insert_item(utxo.clone());

        // Test insertion is successful
        assert!(result.is_ok());

        // Test UTXO is now stored in the tree
        assert_eq!(smt.store.get(&utxo.utxo.hash).unwrap().hash, utxo.utxo.hash);

        // Test curr_root is updated
        assert!(smt.curr_root.is_some());
    }

    #[test]
    fn test_insert_items() {
        let mut smt = UTXOSparseMerkleTree::new();
        let utxo1 = sample_utxo_input(1, 1, 1, 10).unwrap();
        let utxo2 = sample_utxo_input(2, 1, 1, 11).unwrap();

        let result = smt.insert_items(vec![utxo1.clone(), utxo2.clone()]);

        // Test insertion of multiple items is successful
        assert!(result.is_ok());

        // Test UTXOs are now stored in the tree
        assert!(smt.store.get(&utxo1.utxo.hash).is_some());
        assert!(smt.store.get(&utxo2.utxo.hash).is_some());

        // Test curr_root is updated
        assert!(smt.curr_root.is_some());
    }

    #[test]
    fn test_get_item_exists() {
        let mut smt = UTXOSparseMerkleTree::new();
        let utxo = sample_utxo_input(1, 1, 1, 10).unwrap();

        smt.insert_item(utxo.clone()).unwrap();

        // Test that the UTXO can be retrieved by hash
        let retrieved_utxo = smt.get_item(utxo.utxo.hash).unwrap();
        assert!(retrieved_utxo.is_some());
        assert_eq!(retrieved_utxo.unwrap().hash, utxo.utxo.hash);
    }

    #[test]
    fn test_get_item_not_exists() {
        let mut smt = UTXOSparseMerkleTree::new();
        let utxo = sample_utxo_input(1, 1, 1, 10).unwrap();

        // Insert one UTXO and try to fetch a different hash
        smt.insert_item(utxo).unwrap();

        let non_existent_hash = TreeHashType::default();
        let result = smt.get_item(non_existent_hash).unwrap();

        // Test that retrieval for a non-existent UTXO returns None
        assert!(result.is_none());
    }

    #[test]
    fn test_get_membership_proof() {
        let mut smt = UTXOSparseMerkleTree::new();
        let utxo = sample_utxo_input(1, 1, 1, 10).unwrap();

        smt.insert_item(utxo.clone()).unwrap();

        // Fetch membership proof for the inserted UTXO
        let proof = smt.get_membership_proof(utxo.utxo.hash).unwrap();

        // Test proof is generated successfully
        assert!(proof.is_some());
    }

    #[test]
    fn test_get_membership_proof_not_exists() {
        let mut smt = UTXOSparseMerkleTree::new();

        // Try fetching proof for a non-existent UTXO hash
        let non_existent_hash = TreeHashType::default();
        let proof = smt.get_membership_proof(non_existent_hash).unwrap();

        // Test no proof is generated for a non-existent UTXO
        assert!(proof.is_none());
    }
}
