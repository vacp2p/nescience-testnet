use nssa_core::account::AccountWithMetadata;
use risc0_zkvm::guest::env;

fn main() {
    let input_accounts: Vec<AccountWithMetadata> = env::read();
    let balance: u128 = env::read();

    let [sender_pre, receiver_pre] = match input_accounts.try_into() {
        Ok(array) => array,
        Err(_) => return,
    };

    let mut sender_post = sender_pre.account.clone();
    let mut receiver_post = receiver_pre.account.clone();
    sender_post.balance -= balance;
    receiver_post.balance += balance;

    env::commit(&vec![sender_post, receiver_post]);
}

