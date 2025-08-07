use common::{merkle_tree_public::TreeHashType, transaction::AuthenticatedTransaction};
use mempool::mempoolitem::MemPoolItem;

pub struct MempoolTransaction {
    pub auth_tx: nssa::PublicTransaction,
}

impl From<nssa::PublicTransaction> for MempoolTransaction {
    fn from(auth_tx: nssa::PublicTransaction) -> Self {
        Self { auth_tx }
    }
}

impl MemPoolItem for MempoolTransaction {
    type Identifier = TreeHashType;

    fn identifier(&self) -> Self::Identifier {
        self.auth_tx.hash()
    }
}
