mod encoding;
mod message;
mod transaction;
mod witness_set;

pub use transaction::PrivacyPreservingTransaction;

pub mod offchain {
//     use nssa_core::{
//         account::{Account, AccountWithMetadata, NullifierSecretKey}, program::{InstructionData, ProgramOutput}, PrivacyPreservingCircuitInput
//     };
//
//     use crate::{error::NssaError, program::Program};
//
    // pub type Proof = ();
//
//     pub fn execute_offchain(
//         pre_states: &[AccountWithMetadata],
//         instruction_data: &InstructionData,
//         private_account_keys: &[(NullifierSecretKey, ]
//         visibility_mask: &[u8],
//         commitment_set_digest: [u32; 8],
//         program: Program,
//     ) -> Result<(Proof, Vec<Account>), NssaError> {
//         // Prove inner program and get post state of the accounts
//         let inner_proof = program.execute_and_prove(pre_states, instruction_data)?;
//
//         let program_output: ProgramOutput = inner_proof.journal.decode()?;
//
//         // Sample fresh random nonces for the outputs of this execution
//         let output_nonces: Vec<_> = (0..inputs.len()).map(|_| new_random_nonce()).collect();
//
//         let privacy_preserving_circuit_input = PrivacyPreservingCircuitInput {
//             program_output,
//             visibility_mask,
//             private_account_data: todo!(),
//             private_account_auth: todo!(),
//             program_id: todo!(),
//             commitment_set_digest,
//         };
//         //
//         // // Prove outer program.
//         // let mut env_builder = ExecutorEnv::builder();
//         // env_builder.add_assumption(inner_receipt);
//         // env_builder.write(&inner_program_output).unwrap();
//         // env_builder.write(&visibilities).unwrap();
//         // env_builder.write(&output_nonces).unwrap();
//         // env_builder.write(&commitment_tree_root).unwrap();
//         // env_builder.write(&P::PROGRAM_ID).unwrap();
//         // let env = env_builder.build().unwrap();
//         // let prover = default_prover();
//         // let prove_info = prover.prove(env, OUTER_ELF).unwrap();
//         //
//         // // Build private accounts.
//         // let private_outputs = build_private_outputs_from_inner_results(
//         //     &inner_program_output,
//         //     visibilities,
//         //     &output_nonces,
//         // );
//         //
//         // Ok((prove_info.receipt, private_outputs))
//     }
}
