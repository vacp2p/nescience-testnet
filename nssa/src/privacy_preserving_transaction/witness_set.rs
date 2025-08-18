use crate::{privacy_preserving_transaction::message::Message, PrivateKey, PublicKey, Signature};


type Proof = ();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessSet {
    pub(super) signatures_and_public_keys: Vec<(Signature, PublicKey)>,
    pub(super) proof: Proof
}


impl WitnessSet {
    pub fn for_message(message: &Message, private_keys: &[&PrivateKey]) -> Self {
        todo!()
    }

    pub fn signatures_are_valid_for(&self, message: &Message) -> bool {
        todo!()
    }

    pub fn signatures_and_public_keys(&self) -> &[(Signature, PublicKey)] {
        &self.signatures_and_public_keys
    }

    pub fn proof(&self) -> &Proof {
        &self.proof
    }
}
