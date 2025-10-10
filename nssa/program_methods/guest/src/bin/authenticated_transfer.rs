use nssa_core::{
    account::Account,
    program::{ProgramInput, read_nssa_inputs, write_nssa_outputs},
};

/// A transfer of balance program.
/// To be used both in public and private contexts.
fn main() {
    // Read input accounts.
    // It is expected to receive only two accounts: [sender_account, receiver_account]
    let ProgramInput {
        pre_states,
        instruction: balance_to_move,
    } = read_nssa_inputs();

    if pre_states.len() == 1 {
        // Claim account
        let account_to_claim = pre_states[0].account.clone();
        let is_authorized = pre_states[0].is_authorized;
        if account_to_claim == Account::default() && balance_to_move == 0 && is_authorized {
            write_nssa_outputs(pre_states, vec![account_to_claim]);
        } else {
            panic!("Invalid params");
        }
    } else {
        // Transfer

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
}
