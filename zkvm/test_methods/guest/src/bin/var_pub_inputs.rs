// A POC of how a guest program could handle variable number inputs defined as public or private at proving time


use risc0_zkvm::guest::env;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input<T> {
    value: T,
    is_public: bool,
}

fn read_inputs<T>() -> (Vec<T>, Vec<T>)
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    let num_inputs: u64 = env::read();

    let mut pub_inputs = Vec::new();
    let mut priv_inputs = Vec::new();
    for _ in 0..num_inputs {
        let input: Input<T> = env::read();
        if input.is_public {
            pub_inputs.push(input.value);
        } else {
            priv_inputs.push(input.value);
        }
    }
    (priv_inputs, pub_inputs)
}

fn main() {
    let (priv_inputs, pub_inputs) = read_inputs::<u64>();

    let sum = priv_inputs.iter().sum::<u64>() + pub_inputs.iter().sum::<u64>();

    env::commit(&(pub_inputs, sum));
}
