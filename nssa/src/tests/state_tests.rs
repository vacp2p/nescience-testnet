use crate::{
    Address, PublicKey, PublicTransaction, V01State, error::NssaError, program::Program,
    public_transaction, signature::PrivateKey,
};
use nssa_core::account::Account;

fn transfer_transaction(
    from: Address,
    from_key: PrivateKey,
    nonce: u128,
    to: Address,
    balance: u128,
) -> PublicTransaction {
    let addresses = vec![from, to];
    let nonces = vec![nonce];
    let program_id = Program::authenticated_transfer_program().id();
    let message =
        public_transaction::Message::try_new(program_id, addresses, nonces, balance).unwrap();
    let witness_set = public_transaction::WitnessSet::for_message(&message, &[&from_key]);
    PublicTransaction::new(message, witness_set)
}

#[test]
fn transition_from_authenticated_transfer_program_invocation_default_account_destination() {
    let key = PrivateKey::try_new([1; 32]).unwrap();
    let address = Address::from_public_key(&PublicKey::new(&key));
    let initial_data = [(*address.value(), 100)];
    let mut state = V01State::new_with_genesis_accounts(&initial_data);
    let from = address;
    let to = Address::new([2; 32]);
    assert_eq!(state.get_account_by_address(&to), Account::default());
    let balance_to_move = 5;

    let tx = transfer_transaction(from.clone(), key, 0, to.clone(), balance_to_move);
    state.transition_from_public_transaction(&tx).unwrap();

    assert_eq!(state.get_account_by_address(&from).balance, 95);
    assert_eq!(state.get_account_by_address(&to).balance, 5);
    assert_eq!(state.get_account_by_address(&from).nonce, 1);
    assert_eq!(state.get_account_by_address(&to).nonce, 0);
}

#[test]
fn transition_from_authenticated_transfer_program_invocation_insuficient_balance() {
    let key = PrivateKey::try_new([1; 32]).unwrap();
    let address = Address::from_public_key(&PublicKey::new(&key));
    let initial_data = [(*address.value(), 100)];
    let mut state = V01State::new_with_genesis_accounts(&initial_data);
    let from = address;
    let from_key = key;
    let to = Address::new([2; 32]);
    let balance_to_move = 101;
    assert!(state.get_account_by_address(&from).balance < balance_to_move);

    let tx = transfer_transaction(from.clone(), from_key, 0, to.clone(), balance_to_move);
    let result = state.transition_from_public_transaction(&tx);

    assert!(matches!(result, Err(NssaError::ProgramExecutionFailed(_))));
    assert_eq!(state.get_account_by_address(&from).balance, 100);
    assert_eq!(state.get_account_by_address(&to).balance, 0);
    assert_eq!(state.get_account_by_address(&from).nonce, 0);
    assert_eq!(state.get_account_by_address(&to).nonce, 0);
}

#[test]
fn transition_from_authenticated_transfer_program_invocation_non_default_account_destination() {
    let key1 = PrivateKey::try_new([1; 32]).unwrap();
    let key2 = PrivateKey::try_new([2; 32]).unwrap();
    let address1 = Address::from_public_key(&PublicKey::new(&key1));
    let address2 = Address::from_public_key(&PublicKey::new(&key2));
    let initial_data = [(*address1.value(), 100), (*address2.value(), 200)];
    let mut state = V01State::new_with_genesis_accounts(&initial_data);
    let from = address2;
    let from_key = key2;
    let to = address1;
    assert_ne!(state.get_account_by_address(&to), Account::default());
    let balance_to_move = 8;

    let tx = transfer_transaction(from.clone(), from_key, 0, to.clone(), balance_to_move);
    state.transition_from_public_transaction(&tx).unwrap();

    assert_eq!(state.get_account_by_address(&from).balance, 192);
    assert_eq!(state.get_account_by_address(&to).balance, 108);
    assert_eq!(state.get_account_by_address(&from).nonce, 1);
    assert_eq!(state.get_account_by_address(&to).nonce, 0);
}

#[test]
fn transition_from_chained_authenticated_transfer_program_invocations() {
    let key1 = PrivateKey::try_new([8; 32]).unwrap();
    let address1 = Address::from_public_key(&PublicKey::new(&key1));
    let key2 = PrivateKey::try_new([2; 32]).unwrap();
    let address2 = Address::from_public_key(&PublicKey::new(&key2));
    let initial_data = [(*address1.value(), 100)];
    let mut state = V01State::new_with_genesis_accounts(&initial_data);
    let address3 = Address::new([3; 32]);
    let balance_to_move = 5;

    let tx = transfer_transaction(address1.clone(), key1, 0, address2.clone(), balance_to_move);
    state.transition_from_public_transaction(&tx).unwrap();
    let balance_to_move = 3;
    let tx = transfer_transaction(address2.clone(), key2, 0, address3.clone(), balance_to_move);
    state.transition_from_public_transaction(&tx).unwrap();

    assert_eq!(state.get_account_by_address(&address1).balance, 95);
    assert_eq!(state.get_account_by_address(&address2).balance, 2);
    assert_eq!(state.get_account_by_address(&address3).balance, 3);
    assert_eq!(state.get_account_by_address(&address1).nonce, 1);
    assert_eq!(state.get_account_by_address(&address2).nonce, 1);
    assert_eq!(state.get_account_by_address(&address3).nonce, 0);
}
