use sha2::{Digest, digest::FixedOutput};

use crate::{
    Address, PrivateKey, PublicKey, PublicTransaction, Signature, V01State,
    error::NssaError,
    program::Program,
    public_transaction::{Message, WitnessSet},
};

fn keys_for_tests() -> (PrivateKey, PrivateKey, Address, Address) {
    let key1 = PrivateKey::try_new([1; 32]).unwrap();
    let key2 = PrivateKey::try_new([2; 32]).unwrap();
    let addr1 = Address::from_public_key(&PublicKey::new(&key1));
    let addr2 = Address::from_public_key(&PublicKey::new(&key2));
    (key1, key2, addr1, addr2)
}

fn state_for_tests() -> V01State {
    let (key1, key2, addr1, addr2) = keys_for_tests();
    let initial_data = [(*addr1.value(), 10000), (*addr2.value(), 20000)];
    V01State::new_with_genesis_accounts(&initial_data)
}

fn transaction_for_tests() -> PublicTransaction {
    let (key1, key2, addr1, addr2) = keys_for_tests();
    let nonces = vec![0, 0];
    let instruction = 1337;
    let message = Message::try_new(
        Program::authenticated_transfer_program().id(),
        vec![addr1, addr2],
        nonces,
        instruction,
    )
    .unwrap();

    let witness_set = WitnessSet::for_message(&message, &[&key1, &key2]);
    PublicTransaction::new(message, witness_set)
}

#[test]
fn test_new_constructor() {
    let tx = transaction_for_tests();
    let message = tx.message().clone();
    let witness_set = tx.witness_set().clone();
    let tx_from_constructor = PublicTransaction::new(message.clone(), witness_set.clone());
    assert_eq!(tx_from_constructor.message, message);
    assert_eq!(tx_from_constructor.witness_set, witness_set);
}

#[test]
fn test_message_getter() {
    let tx = transaction_for_tests();
    assert_eq!(&tx.message, tx.message());
}

#[test]
fn test_witness_set_getter() {
    let tx = transaction_for_tests();
    assert_eq!(&tx.witness_set, tx.witness_set());
}

#[test]
fn test_signer_addresses() {
    let tx = transaction_for_tests();
    let expected_signer_addresses = vec![
        Address::new([
            27, 132, 197, 86, 123, 18, 100, 64, 153, 93, 62, 213, 170, 186, 5, 101, 215, 30, 24,
            52, 96, 72, 25, 255, 156, 23, 245, 233, 213, 221, 7, 143,
        ]),
        Address::new([
            77, 75, 108, 209, 54, 16, 50, 202, 155, 210, 174, 185, 217, 0, 170, 77, 69, 217, 234,
            216, 10, 201, 66, 51, 116, 196, 81, 167, 37, 77, 7, 102,
        ]),
    ];
    let signer_addresses = tx.signer_addresses();
    assert_eq!(signer_addresses, expected_signer_addresses);
}

#[test]
fn test_public_transaction_encoding_bytes_roundtrip() {
    let tx = transaction_for_tests();
    let bytes = tx.to_bytes();
    let tx_from_bytes = PublicTransaction::from_bytes(&bytes).unwrap();
    assert_eq!(tx, tx_from_bytes);
}

#[test]
fn test_hash_is_sha256_of_transaction_bytes() {
    let tx = transaction_for_tests();
    let hash = tx.hash();
    let expected_hash: [u8; 32] = {
        let bytes = tx.to_bytes();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&bytes);
        hasher.finalize_fixed().into()
    };
    assert_eq!(hash, expected_hash);
}

#[test]
fn test_address_list_cant_have_duplicates() {
    let (key1, _, addr1, _) = keys_for_tests();
    let state = state_for_tests();
    let nonces = vec![0, 0];
    let instruction = 1337;
    let message = Message::try_new(
        Program::authenticated_transfer_program().id(),
        vec![addr1.clone(), addr1],
        nonces,
        instruction,
    )
    .unwrap();

    let witness_set = WitnessSet::for_message(&message, &[&key1, &key1]);
    let tx = PublicTransaction::new(message, witness_set);
    let result = tx.validate_and_compute_post_states(&state);
    assert!(matches!(result, Err(NssaError::InvalidInput(_))))
}

#[test]
fn test_number_of_nonces_must_match_number_of_signatures() {
    let (key1, key2, addr1, addr2) = keys_for_tests();
    let state = state_for_tests();
    let nonces = vec![0];
    let instruction = 1337;
    let message = Message::try_new(
        Program::authenticated_transfer_program().id(),
        vec![addr1, addr2],
        nonces,
        instruction,
    )
    .unwrap();

    let witness_set = WitnessSet::for_message(&message, &[&key1, &key2]);
    let tx = PublicTransaction::new(message, witness_set);
    let result = tx.validate_and_compute_post_states(&state);
    assert!(matches!(result, Err(NssaError::InvalidInput(_))))
}

#[test]
fn test_all_signatures_must_be_valid() {
    let (key1, key2, addr1, addr2) = keys_for_tests();
    let state = state_for_tests();
    let nonces = vec![0, 0];
    let instruction = 1337;
    let message = Message::try_new(
        Program::authenticated_transfer_program().id(),
        vec![addr1, addr2],
        nonces,
        instruction,
    )
    .unwrap();

    let mut witness_set = WitnessSet::for_message(&message, &[&key1, &key2]);
    witness_set.signatures_and_public_keys[0].0 = Signature { value: [1; 64] };
    let tx = PublicTransaction::new(message, witness_set);
    let result = tx.validate_and_compute_post_states(&state);
    assert!(matches!(result, Err(NssaError::InvalidInput(_))))
}

#[test]
fn test_nonces_must_match_the_state_current_nonces() {
    let (key1, key2, addr1, addr2) = keys_for_tests();
    let state = state_for_tests();
    let nonces = vec![0, 1];
    let instruction = 1337;
    let message = Message::try_new(
        Program::authenticated_transfer_program().id(),
        vec![addr1, addr2],
        nonces,
        instruction,
    )
    .unwrap();

    let witness_set = WitnessSet::for_message(&message, &[&key1, &key2]);
    let tx = PublicTransaction::new(message, witness_set);
    let result = tx.validate_and_compute_post_states(&state);
    assert!(matches!(result, Err(NssaError::InvalidInput(_))))
}

#[test]
fn test_program_id_must_belong_to_bulitin_program_ids() {
    let (key1, key2, addr1, addr2) = keys_for_tests();
    let state = state_for_tests();
    let nonces = vec![0, 0];
    let instruction = 1337;
    let unknown_program_id = [0xdeadbeef; 8];
    let message =
        Message::try_new(unknown_program_id, vec![addr1, addr2], nonces, instruction).unwrap();

    let witness_set = WitnessSet::for_message(&message, &[&key1, &key2]);
    let tx = PublicTransaction::new(message, witness_set);
    let result = tx.validate_and_compute_post_states(&state);
    assert!(matches!(result, Err(NssaError::InvalidInput(_))))
}
