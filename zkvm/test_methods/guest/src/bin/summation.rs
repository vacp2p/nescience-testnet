use risc0_zkvm::{
    guest::env,
};

fn main() {
    let data: u64 = env::read();
    let data_2: u64 = env::read();
    env::commit(&(data + data_2));
}
