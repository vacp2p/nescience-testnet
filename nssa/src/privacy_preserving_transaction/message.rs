use nssa_core::{
    CommitmentSetDigest, EncryptedAccountData,
    account::{Account, Commitment, Nonce, Nullifier},
};

use crate::Address;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub(crate) public_addresses: Vec<Address>,
    pub(crate) nonces: Vec<Nonce>,
    pub(crate) public_post_states: Vec<Account>,
    pub(crate) encrypted_private_post_states: Vec<EncryptedAccountData>,
    pub(crate) new_commitments: Vec<Commitment>,
    pub(crate) new_nullifiers: Vec<(Nullifier, CommitmentSetDigest)>,
}

impl Message {
    pub fn new(
        public_addresses: Vec<Address>,
        nonces: Vec<Nonce>,
        public_post_states: Vec<Account>,
        encrypted_private_post_states: Vec<EncryptedAccountData>,
        new_commitments: Vec<Commitment>,
        new_nullifiers: Vec<(Nullifier, CommitmentSetDigest)>,
    ) -> Self {
        Self {
            public_addresses,
            nonces,
            public_post_states,
            encrypted_private_post_states,
            new_commitments,
            new_nullifiers,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::io::Cursor;

    use nssa_core::account::{
        Account, Commitment, Nullifier, NullifierPublicKey, NullifierSecretKey,
    };

    use crate::{Address, privacy_preserving_transaction::message::Message};

    pub fn message_for_tests() -> Message {
        let account1 = Account::default();
        let account2 = Account::default();

        let nsk1 = [11; 32];
        let nsk2 = [12; 32];

        let Npk1 = NullifierPublicKey::from(&nsk1);
        let Npk2 = NullifierPublicKey::from(&nsk2);

        let public_addresses = vec![Address::new([1; 32])];

        let nonces = vec![1, 2, 3];

        let public_post_states = vec![Account::default()];

        let encrypted_private_post_states = Vec::new();

        let new_commitments = vec![Commitment::new(&Npk2, &account2)];

        let old_commitment = Commitment::new(&Npk1, &account1);
        let new_nullifiers = vec![(Nullifier::new(&old_commitment, &nsk1), [0; 32])];

        Message {
            public_addresses: public_addresses.clone(),
            nonces: nonces.clone(),
            public_post_states: public_post_states.clone(),
            encrypted_private_post_states: encrypted_private_post_states.clone(),
            new_commitments: new_commitments.clone(),
            new_nullifiers: new_nullifiers.clone(),
        }
    }

    #[test]
    fn test_constructor() {
        let message = message_for_tests();
        let expected_message = message.clone();

        let message = Message::new(
            message.public_addresses,
            message.nonces,
            message.public_post_states,
            message.encrypted_private_post_states,
            message.new_commitments,
            message.new_nullifiers,
        );

        assert_eq!(message, expected_message);
    }

    #[test]
    fn test_message_serialization_roundtrip() {
        let message = message_for_tests();

        let bytes = message.to_bytes();
        let mut cursor = Cursor::new(bytes.as_ref());
        let message_from_cursor = Message::from_cursor(&mut cursor).unwrap();

        assert_eq!(message, message_from_cursor);
    }
}
