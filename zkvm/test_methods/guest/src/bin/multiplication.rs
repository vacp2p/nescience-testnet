use risc0_zkvm::{
    guest::env,
};

fn main() {
    let lhs: u64 = env::read();
    let rhs: u64 = env::read();
    env::commit(&(lhs * rhs));
}
