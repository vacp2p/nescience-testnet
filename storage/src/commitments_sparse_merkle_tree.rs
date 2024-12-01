use monotree::database::MemoryDB;
use monotree::hasher::Blake3;
use monotree::{Hasher, Monotree, Proof};

use crate::commitment::Commitment;
use crate::merkle_tree_public::CommitmentHashType;
use crate::nullifier::UTXONullifier;

pub struct CommitmentsSparseMerkleTree {
    pub curr_root: Option<CommitmentHashType>,
    pub tree: Monotree<MemoryDB, Blake3>,
    pub hasher: Blake3,
}

impl CommitmentsSparseMerkleTree {
    pub fn new() -> Self {
        CommitmentsSparseMerkleTree {
            curr_root: None,
            tree: Monotree::default(),
            hasher: Blake3::new(),
        }
    }

    pub fn insert_item(&mut self, commitment: Commitment) -> Result<(), monotree::Errors> {
        let root = self.curr_root.as_ref().map(|val | val[0..32].try_into().unwrap());

        let new_root = self
            .tree
            .insert(root, &commitment.commitment_hash[0..32].try_into().unwrap(), &commitment.commitment_hash[0..32].try_into().unwrap())?;

        self.curr_root = new_root.map(|val| val.to_vec());

        Ok(())
    }

    pub fn insert_items(&mut self, commitments: Vec<Commitment>) -> Result<(), monotree::Errors> {
        let root = self.curr_root.as_ref().map(|val | val[0..32].try_into().unwrap());

        let hashes: Vec<_> = commitments.iter().map(|val | val.commitment_hash[0..32].try_into().unwrap()).collect::<Vec<_>>();

        let new_root = self.tree.inserts(root, &hashes, &hashes)?;

        self.curr_root = new_root.map(|val | val[0..32].try_into().unwrap());

        Ok(())
    }

    pub fn search_item_inclusion(
        &mut self,
        commitment_hash: CommitmentHashType,
    ) -> Result<bool, monotree::Errors> {
        self.tree
            .get(self.curr_root.as_ref().map(|val | val[0..32].try_into().unwrap()), &commitment_hash[0..32].try_into().unwrap())
            .map(|data| data.is_some())
    }

    pub fn search_item_inclusions(
        &mut self,
        commitment_hashes: &[CommitmentHashType],
    ) -> Result<Vec<bool>, monotree::Errors> {
        let mut inclusions = vec![];

        for nullifier_hash in commitment_hashes {
            let is_included = self
                .tree
                .get(self.curr_root.as_ref().map(|val | val[0..32].try_into().unwrap()), nullifier_hash[0..32].try_into().unwrap())
                .map(|data| data.is_some())?;

            inclusions.push(is_included);
        }

        Ok(inclusions)
    }

    pub fn get_non_membership_proof(
        &mut self,
        commitment_hash: CommitmentHashType,
    ) -> Result<(Option<Proof>, Option<CommitmentHashType>), monotree::Errors> {
        let is_member = self.search_item_inclusion(commitment_hash.clone())?;

        if is_member {
            Err(monotree::Errors::new("Is a member"))
        } else {
            Ok((
                self.tree
                    .get_merkle_proof(self.curr_root.as_ref().map(|val | val[0..32].try_into().unwrap()), &commitment_hash)?,
                self.curr_root.clone(),
            ))
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn get_non_membership_proofs(
        &mut self,
        commitment_hashes: &[CommitmentHashType],
    ) -> Result<Vec<(Option<Proof>, Option<CommitmentHashType>)>, monotree::Errors> {
        let mut non_membership_proofs = vec![];

        for commitment_hash in commitment_hashes {
            let is_member = self.search_item_inclusion(commitment_hash.clone())?;

            if is_member {
                return Err(monotree::Errors::new(
                    format!("{commitment_hash:?} Is a member").as_str(),
                ));
            } else {
                non_membership_proofs.push((
                    self.tree
                        .get_merkle_proof(self.curr_root.as_ref().map(|val | val[0..32].try_into().unwrap()), commitment_hash)?,
                    self.curr_root.clone(),
                ))
            };
        }

        Ok(non_membership_proofs)
    }
}

impl Default for CommitmentsSparseMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}
