use risc0_zkvm::{guest::env, serde::to_vec};

use nssa_core::{
    account::{Account, AccountWithMetadata, Commitment, Nullifier, NullifierPublicKey},
    program::{validate_execution, ProgramOutput},
    verify_membership_proof, EncryptedAccountData, EphemeralPublicKey, EphemeralSecretKey,
    IncomingViewingPublicKey, PrivacyPreservingCircuitInput, PrivacyPreservingCircuitOutput, Tag,
};

fn main() {
    let PrivacyPreservingCircuitInput {
        program_output,
        visibility_mask,
        private_account_data,
        private_account_auth,
        program_id,
        commitment_set_digest,
    } = env::read();

    // TODO: Check that `program_execution_proof` is one of the allowed built-in programs
    // assert!(BUILTIN_PROGRAM_IDS.contains(executing_program_id));

    // Check that `program_output` is consistent with the execution of the corresponding program.
    env::verify(program_id, &to_vec(&program_output).unwrap()).unwrap();

    let ProgramOutput {
        pre_states,
        post_states,
    } = program_output;

    // Check that the program is well behaved.
    // See the # Programs section for the definition of the `validate_execution` method.
    validate_execution(&pre_states, &post_states, program_id);

    let n_accounts = pre_states.len();
    assert_eq!(visibility_mask.len(), n_accounts);

    let n_private_accounts = visibility_mask.iter().filter(|&&flag| flag != 0).count();
    assert_eq!(private_account_data.len(), n_private_accounts);

    let n_auth_private_accounts = visibility_mask.iter().filter(|&&flag| flag == 1).count();
    assert_eq!(private_account_auth.len(), n_auth_private_accounts);

    // These lists will be the public outputs of this circuit
    // and will be populated next.
    let mut public_pre_states: Vec<AccountWithMetadata> = Vec::new();
    let mut public_post_states: Vec<Account> = Vec::new();
    let mut encrypted_private_post_states: Vec<EncryptedAccountData> = Vec::new();
    let mut new_commitments: Vec<Commitment> = Vec::new();
    let mut new_nullifiers: Vec<Nullifier> = Vec::new();

    for i in 0..n_accounts {
        // visibility_mask[i] equal to 0 means public
        if visibility_mask[i] == 0 {
            // If the account is marked as public, add the pre and post
            // states to the corresponding lists.
            public_pre_states.push(pre_states[i].clone());
            public_post_states.push(post_states[i].clone());
        } else {
            let (new_nonce, Npk, Ipk, esk) = &private_account_data[i];

            // Verify authentication
            if visibility_mask[i] == 1 {
                let (nsk, membership_proof) = &private_account_auth[i];

                // 1. Compute Npk from the provided nsk and assert it is equal to the provided Npk
                let expected_Npk = NullifierPublicKey::from(nsk);
                assert_eq!(&expected_Npk, Npk);
                // 2. Compute the commitment of the pre_state account using the provided Npk
                let commitment_pre = Commitment::new(Npk, &pre_states[i].account);
                // 3. Verify that the commitment belongs to the global commitment set
                assert!(verify_membership_proof(
                    &commitment_pre,
                    membership_proof,
                    &commitment_set_digest,
                ));
                // At this point the account is correctly authenticated as a private account.
                // Assert that `pre_states` marked this account as authenticated.
                assert!(pre_states[i].is_authorized);
                // Compute the nullifier of the pre state version of this private account
                // and include it in the `new_nullifiers` list.
                let nullifier = Nullifier::new(&commitment_pre, nsk);
                new_nullifiers.push(nullifier);
            } else if visibility_mask[i] == 2 {
                assert_eq!(pre_states[i].account, Account::default());
                assert!(!pre_states[i].is_authorized);
            } else {
                panic!();
            }

            // Update the nonce for the post state of this private account.
            let mut post_with_updated_nonce = post_states[i].clone();
            post_with_updated_nonce.nonce = *new_nonce;

            // Compute the commitment of the post state of the private account,
            // with the updated nonce, and include it in the `new_commitments` list.
            let commitment_post = Commitment::new(Npk, &post_with_updated_nonce);
            new_commitments.push(commitment_post);

            // Encrypt the post state of the private account with the updated
            // nonce and include it in the `encrypted_private_post_states` list.
            //
            let encrypted_account = EncryptedAccountData::new(&post_with_updated_nonce, esk, Npk, Ipk);
            encrypted_private_post_states.push(encrypted_account);
        }
    }

    let output = PrivacyPreservingCircuitOutput {
        public_pre_states,
        public_post_states,
        encrypted_private_post_states,
        new_commitments,
        new_nullifiers,
        commitment_set_digest,
    };

    env::commit(&output);
}
