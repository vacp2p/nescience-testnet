use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{ProgramInput, read_nssa_inputs, write_nssa_outputs},
};

fn initialize_account(pre_states: Vec<AccountWithMetadata>) {
    // Continue only if input_accounts is an array of one element
    let [pre_state] = match pre_states.try_into() {
        Ok(array) => array,
        Err(_) => return,
    };
    let account_to_claim = pre_state.account.clone();
    let is_authorized = pre_state.is_authorized;

    // Continue only if the account to claim has default values
    if account_to_claim != Account::default() {
        return;
    }

    // Continue only if the owner authorized this operation
    if !is_authorized {
        return;
    }

    // Noop will result in account being claimed for this program
    write_nssa_outputs(vec![pre_state], vec![account_to_claim]);
}

fn transfer(pre_states: Vec<AccountWithMetadata>, balance_to_move: u128) {
    // Continue only if input_accounts is an array of two elements
    let [sender, receiver] = match pre_states.try_into() {
        Ok(array) => array,
        Err(_) => return,
    };

    // Continue only if the sender has authorized this operation
    if !sender.is_authorized {
        return;
    }

    // Continue only if the sender has enough balance
    if sender.account.balance < balance_to_move {
        return;
    }

    // Create accounts post states, with updated balances
    let mut sender_post = sender.account.clone();
    let mut receiver_post = receiver.account.clone();
    sender_post.balance -= balance_to_move;
    receiver_post.balance += balance_to_move;

    write_nssa_outputs(vec![sender, receiver], vec![sender_post, receiver_post]);
}

/// A transfer of balance program.
/// To be used both in public and private contexts.
fn main() {
    // Read input accounts.
    // It is expected to receive only two accounts: [sender_account, receiver_account]
    let ProgramInput {
        pre_states,
        instruction: balance_to_move,
    } = read_nssa_inputs();

    match (pre_states.len(), balance_to_move) {
        (1, 0) => initialize_account(pre_states),
        (2, balance_to_move) => transfer(pre_states, balance_to_move),
        _ => panic!("Invalid parameters"),
    }
}
