use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::ProgramId,
};
use program_methods::{AUTHENTICATED_TRANSFER_ELF, AUTHENTICATED_TRANSFER_ID};
use risc0_zkvm::{ExecutorEnv, ExecutorEnvBuilder, default_executor};

mod address;
pub mod public_transaction;
mod signature;
mod state;

pub use address::Address;
pub use nssa_core::program::Program;
pub use public_transaction::PublicTransaction;
pub use signature::PrivateKey;
pub use state::V01State;

pub struct AuthenticatedTransferProgram;
impl Program for AuthenticatedTransferProgram {
    const PROGRAM_ID: ProgramId = AUTHENTICATED_TRANSFER_ID;
    const PROGRAM_ELF: &[u8] = AUTHENTICATED_TRANSFER_ELF;
    type InstructionData = u128;
}

/// Writes inputs to `env_builder` in the order expected by the programs
fn write_inputs<P: Program>(
    pre_states: &[AccountWithMetadata],
    instruction_data: P::InstructionData,
    env_builder: &mut ExecutorEnvBuilder,
) -> Result<(), ()> {
    let pre_states = pre_states.to_vec();
    env_builder.write(&pre_states).map_err(|_| ())?;
    env_builder.write(&instruction_data).map_err(|_| ())?;
    Ok(())
}

fn execute_public<P: Program>(
    pre_states: &[AccountWithMetadata],
    instruction_data: P::InstructionData,
) -> Result<Vec<Account>, ()> {
    // Write inputs to the program
    let mut env_builder = ExecutorEnv::builder();
    write_inputs::<P>(pre_states, instruction_data, &mut env_builder)?;
    let env = env_builder.build().unwrap();

    // Execute the program (without proving)
    let executor = default_executor();
    let session_info = executor.execute(env, P::PROGRAM_ELF).map_err(|_| ())?;

    // Get (inputs and) outputs
    session_info.journal.decode().map_err(|_| ())
}
