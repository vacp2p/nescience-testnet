use nssa_core::account::{Account, AccountWithMetadata};
use risc0_zkvm::guest::env;

fn main() {
    let input_accounts: Vec<AccountWithMetadata> = env::read();
    let _instruction_data: u128 = env::read();

    let [pre] = match input_accounts.try_into() {
        Ok(array) => array,
        Err(_) => return,
    };

    let account_pre = pre.account;

    env::commit(&vec![account_pre, Account::default()]);
}

