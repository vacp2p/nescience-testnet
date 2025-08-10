use crate::{
    Address, PublicTransaction, V01State, error::NssaError, program::Program, public_transaction,
};

#[test]
fn test_program_should_fail_if_it_modifies_nonces() {
    let initial_data = [([1; 32], 100)];
    let mut state = V01State::new_with_genesis_accounts(&initial_data).with_test_programs();
    let addresses = vec![Address::new([1; 32])];
    let nonces = vec![];
    let program_id = Program::nonce_changer_program().id();
    let message = public_transaction::Message::new(program_id, addresses, nonces, 0);
    let witness_set = public_transaction::WitnessSet::for_message(&message, &[]);
    let tx = PublicTransaction::new(message, witness_set);

    let result = state.transition_from_public_transaction(&tx);

    assert!(matches!(result, Err(NssaError::InvalidProgramBehavior)));
}

#[test]
fn test_program_should_fail_if_output_accounts_exceed_inputs() {
    let initial_data = [([1; 32], 100)];
    let mut state = V01State::new_with_genesis_accounts(&initial_data).with_test_programs();
    let addresses = vec![Address::new([1; 32])];
    let nonces = vec![];
    let program_id = Program::extra_outputs_program().id();
    let message = public_transaction::Message::new(program_id, addresses, nonces, 0);
    let witness_set = public_transaction::WitnessSet::for_message(&message, &[]);
    let tx = PublicTransaction::new(message, witness_set);

    let result = state.transition_from_public_transaction(&tx);

    assert!(matches!(result, Err(NssaError::InvalidProgramBehavior)));
}
