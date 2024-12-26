use risc0_zkvm::{
    guest::env,
};
use serde::{Deserialize, Serialize};

type AccountAddr = [u8; 32];

#[derive(Serialize, Deserialize)]
pub struct UTXOPayload {
    pub owner: AccountAddr,
    pub asset: Vec<u8>,
    // TODO: change to u256
    pub amount: u128,
    pub privacy_flag: bool,
}

fn main() {
    let amount_to_mint: u128 = env::read();
    let owner: AccountAddr = env::read();

    let payload = UTXOPayload {
        owner,
        asset: vec![],
        amount: amount_to_mint,
        privacy_flag: true,
    };

    env::commit(&(payload));
}
