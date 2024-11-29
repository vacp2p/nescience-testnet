use serde::{Deserialize, Serialize};

use crate::merkle_tree_public::TreeHashType;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum TxKind {
    Public,
    Private,
    Shielded,
    Deshielded,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
///General transaction object
pub struct Transaction {
    pub hash: TreeHashType,
    pub tx_kind: TxKind,
    ///Tx input data (public part)
    pub execution_input: Vec<u8>,
    ///Tx output data (public_part)
    pub execution_output: Vec<u8>,
    ///Tx output utxo commitments
    pub utxo_commitments_created_hashes: Vec<TreeHashType>,
    ///Tx output nullifiers
    pub nullifier_created_hashes: Vec<TreeHashType>,
    ///Execution proof (private part)
    pub execution_proof_private: String,
}
