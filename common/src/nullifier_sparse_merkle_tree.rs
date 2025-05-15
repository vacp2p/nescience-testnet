use std::collections::BTreeMap;

use monotree::database::MemoryDB;
use monotree::hasher::Blake3;
use monotree::{Hasher, Monotree, Proof};

use crate::merkle_tree_public::TreeHashType;
use crate::nullifier::UTXONullifier;

#[derive(Debug, Clone)]
pub struct NullifierTreeInput {
    pub nullifier_id: u64,
    pub tx_id: u64,
    pub block_id: u64,
    pub nullifier: UTXONullifier,
}

#[derive(Debug, Clone)]
pub struct TreeTxWithNullifierId {
    pub id: u64,
    pub nullifiers: BTreeMap<u64, UTXONullifier>,
}

#[derive(Debug, Clone)]
pub struct TreeBlockWithTxId {
    pub id: u64,
    pub txs: BTreeMap<u64, TreeTxWithNullifierId>,
}

pub struct NullifierSparseMerkleTree {
    pub curr_root: Option<TreeHashType>,
    pub tree: Monotree<MemoryDB, Blake3>,
    pub hasher: Blake3,
    pub leafs: BTreeMap<u64, TreeBlockWithTxId>,
}

impl NullifierSparseMerkleTree {
    pub fn new() -> Self {
        NullifierSparseMerkleTree {
            curr_root: None,
            tree: Monotree::default(),
            hasher: Blake3::new(),
            leafs: BTreeMap::new(),
        }
    }

    pub fn modify_leavs_with_nullifier_input(&mut self, tree_nullifier: NullifierTreeInput) {
        self.leafs
            .entry(tree_nullifier.block_id)
            .and_modify(|tree_block| {
                tree_block
                    .txs
                    .entry(tree_nullifier.tx_id)
                    .and_modify(|tree_tx| {
                        tree_tx
                            .nullifiers
                            .insert(tree_nullifier.nullifier_id, tree_nullifier.nullifier);
                    })
                    .or_insert(TreeTxWithNullifierId {
                        id: tree_nullifier.tx_id,
                        nullifiers: BTreeMap::new(),
                    });
            })
            .or_insert(TreeBlockWithTxId {
                id: tree_nullifier.block_id,
                txs: BTreeMap::new(),
            });
    }

    pub fn insert_item(
        &mut self,
        tree_nullifier: NullifierTreeInput,
    ) -> Result<(), monotree::Errors> {
        let root = self.curr_root.as_ref();

        let new_root = self.tree.insert(
            root,
            &tree_nullifier.nullifier.utxo_hash,
            &tree_nullifier.nullifier.utxo_hash,
        )?;

        self.curr_root = new_root;

        self.modify_leavs_with_nullifier_input(tree_nullifier);

        Ok(())
    }

    pub fn insert_items(
        &mut self,
        tree_nullifiers: Vec<NullifierTreeInput>,
    ) -> Result<(), monotree::Errors> {
        let root = self.curr_root.as_ref();

        let hashes: Vec<TreeHashType> = tree_nullifiers
            .iter()
            .map(|nu| nu.nullifier.utxo_hash)
            .collect();

        let new_root = self.tree.inserts(root, &hashes, &hashes)?;

        self.curr_root = new_root;

        for tree_nullifier in tree_nullifiers {
            self.modify_leavs_with_nullifier_input(tree_nullifier);
        }

        Ok(())
    }

    pub fn search_item_inclusion(
        &mut self,
        nullifier_hash: TreeHashType,
    ) -> Result<bool, monotree::Errors> {
        self.tree
            .get(self.curr_root.as_ref(), &nullifier_hash)
            .map(|data| data.is_some())
    }

    pub fn search_item_inclusions(
        &mut self,
        nullifier_hashes: &[TreeHashType],
    ) -> Result<Vec<bool>, monotree::Errors> {
        let mut inclusions = vec![];

        for nullifier_hash in nullifier_hashes {
            let is_included = self
                .tree
                .get(self.curr_root.as_ref(), nullifier_hash)
                .map(|data| data.is_some())?;

            inclusions.push(is_included);
        }

        Ok(inclusions)
    }

    pub fn get_non_membership_proof(
        &mut self,
        nullifier_hash: TreeHashType,
    ) -> Result<(Option<Proof>, Option<TreeHashType>), monotree::Errors> {
        let is_member = self.search_item_inclusion(nullifier_hash)?;

        if is_member {
            Err(monotree::Errors::new("Is a member"))
        } else {
            Ok((
                self.tree
                    .get_merkle_proof(self.curr_root.as_ref(), &nullifier_hash)?,
                self.curr_root,
            ))
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn get_non_membership_proofs(
        &mut self,
        nullifier_hashes: &[TreeHashType],
    ) -> Result<Vec<(Option<Proof>, Option<TreeHashType>)>, monotree::Errors> {
        let mut non_membership_proofs = vec![];

        for nullifier_hash in nullifier_hashes {
            let is_member = self.search_item_inclusion(*nullifier_hash)?;

            if is_member {
                return Err(monotree::Errors::new(
                    format!("{nullifier_hash:?} Is a member").as_str(),
                ));
            } else {
                non_membership_proofs.push((
                    self.tree
                        .get_merkle_proof(self.curr_root.as_ref(), nullifier_hash)?,
                    self.curr_root,
                ))
            };
        }

        Ok(non_membership_proofs)
    }
}

impl Default for NullifierSparseMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nullifier::UTXONullifier;

    fn create_nullifier(hash: TreeHashType) -> UTXONullifier {
        UTXONullifier { utxo_hash: hash }
    }

    fn create_nullifier_input(
        hash: TreeHashType,
        nullifier_id: u64,
        tx_id: u64,
        block_id: u64,
    ) -> NullifierTreeInput {
        NullifierTreeInput {
            nullifier_id,
            tx_id,
            block_id,
            nullifier: create_nullifier(hash),
        }
    }

    #[test]
    fn test_new_tree_initialization() {
        let tree = NullifierSparseMerkleTree::new();
        assert!(tree.curr_root.is_none());
    }

    #[test]
    fn test_insert_single_item() {
        let mut tree = NullifierSparseMerkleTree::new();
        let tree_nullifier = create_nullifier_input([1u8; 32], 1, 1, 1); // Sample 32-byte hash

        let result = tree.insert_item(tree_nullifier);
        assert!(result.is_ok());
        assert!(tree.curr_root.is_some());
    }

    #[test]
    fn test_insert_multiple_items() {
        let mut tree = NullifierSparseMerkleTree::new();
        let tree_nullifiers = vec![
            create_nullifier_input([1u8; 32], 1, 1, 1),
            create_nullifier_input([2u8; 32], 2, 1, 1),
            create_nullifier_input([3u8; 32], 3, 1, 1),
        ];

        let result = tree.insert_items(tree_nullifiers);
        assert!(result.is_ok());
        assert!(tree.curr_root.is_some());
    }

    #[test]
    fn test_search_item_inclusion() {
        let mut tree = NullifierSparseMerkleTree::new();
        let tree_nullifier = create_nullifier_input([1u8; 32], 1, 1, 1);

        tree.insert_item(tree_nullifier.clone()).unwrap();

        let result = tree.search_item_inclusion([1u8; 32]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        let non_existing = tree.search_item_inclusion([99u8; 32]);
        assert!(non_existing.is_ok());
        assert_eq!(non_existing.unwrap(), false);
    }

    #[test]
    fn test_search_multiple_item_inclusions() {
        let mut tree = NullifierSparseMerkleTree::new();
        let tree_nullifiers = vec![
            create_nullifier_input([1u8; 32], 1, 1, 1),
            create_nullifier_input([2u8; 32], 2, 1, 1),
            create_nullifier_input([3u8; 32], 3, 1, 1),
        ];

        tree.insert_items(tree_nullifiers).unwrap();

        let search_hashes = vec![[1u8; 32], [2u8; 32], [99u8; 32]];
        let result = tree.search_item_inclusions(&search_hashes);
        assert!(result.is_ok());

        let expected_results = vec![true, true, false];
        assert_eq!(result.unwrap(), expected_results);
    }

    #[test]
    fn test_non_membership_proof() {
        let mut tree = NullifierSparseMerkleTree::new();
        let non_member_hash = [5u8; 32];

        let result = tree.get_non_membership_proof(non_member_hash);
        assert!(result.is_ok());

        let (proof, root) = result.unwrap();
        assert!(root.is_none());
    }

    #[test]
    fn test_non_membership_proofs_multiple() {
        let mut tree = NullifierSparseMerkleTree::new();
        let non_member_hashes = vec![[5u8; 32], [6u8; 32], [7u8; 32]];

        let result = tree.get_non_membership_proofs(&non_member_hashes);
        assert!(result.is_ok());

        let proofs = result.unwrap();
        for (proof, root) in proofs {
            assert!(root.is_none());
        }
    }

    #[test]
    fn test_insert_and_get_proof_of_existing_item() {
        let mut tree = NullifierSparseMerkleTree::new();
        let tree_nullifier = create_nullifier_input([1u8; 32], 1, 1, 1);

        tree.insert_item(tree_nullifier.clone()).unwrap();

        let proof_result = tree.get_non_membership_proof([1u8; 32]);
        assert!(proof_result.is_err());
    }

    #[test]
    fn test_insert_and_get_proofs_of_existing_items() {
        let mut tree = NullifierSparseMerkleTree::new();
        let tree_nullifiers = vec![
            create_nullifier_input([1u8; 32], 1, 1, 1),
            create_nullifier_input([2u8; 32], 2, 1, 1),
        ];

        tree.insert_items(tree_nullifiers).unwrap();

        let proof_result = tree.get_non_membership_proofs(&[[1u8; 32], [2u8; 32]]);
        assert!(proof_result.is_err());
    }
}
