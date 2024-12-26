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
    let utxo_spent: UTXOPayload = env::read();
    let owners_parts: Vec<(u128, AccountAddr)> = env::read();

    let res: Vec<(UTXOPayload, AccountAddr)> = owners_parts.into_iter().map(|(amount, addr)| (
        UTXOPayload {
            owner: addr.clone(),
            asset: vec![],
            amount,
            privacy_flag: true,
        },
        addr
    )).collect();

    env::commit(&(res));
}
