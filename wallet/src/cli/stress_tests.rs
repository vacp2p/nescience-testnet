use std::fs::File;

use anyhow::Result;
use clap::Subcommand;
use key_protocol::key_management::ephemeral_key_holder::EphemeralKeyHolder;
use nssa::{
    Account, AccountId, Address, PrivacyPreservingTransaction, PrivateKey, PublicKey,
    PublicTransaction,
    privacy_preserving_transaction::{self as pptx, circuit},
    program::Program,
    public_transaction as putx,
};
use nssa_core::{
    Commitment, MembershipProof, NullifierPublicKey, NullifierSecretKey, SharedSecretKey,
    account::AccountWithMetadata, encryption::IncomingViewingPublicKey,
};
use sequencer_core::config::{AccountInitialData, CommitmentsInitialData, SequencerConfig};

use crate::{SubcommandReturnValue, WalletCore, cli::WalletSubcommand};

///Represents generic chain CLI subcommand
#[derive(Subcommand, Debug, Clone)]
pub enum StressTestSubcommand {
    Run,
    GenerateConfig,
}

impl WalletSubcommand for StressTestSubcommand {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            StressTestSubcommand::Run => {
                println!("Stress test begin");
                // let privacy_tx = build_privacy_tx();
                let txs = build_public_txs();
                for (i, tx) in txs.into_iter().enumerate() {
                    wallet_core.sequencer_client.send_tx_public(tx).await;
                    println!("Sent tx: {}", i);
                    // wallet_core
                    //     .sequencer_client
                    //     .send_tx_private(privacy_tx.clone())
                    //     .await;
                    // println!("Sent tx: {}", i);
                }

                println!("Stress test end");
            }
            StressTestSubcommand::GenerateConfig => {
                println!("Generating config");
                let config = generate_stress_test_config();
                let file = File::create("config.json")?; // crea/trunca el archivo
                serde_json::to_writer_pretty(file, &config).unwrap(); // escribe JSON formateado
                println!("Done");
            }
        }
        Ok(SubcommandReturnValue::Empty)
    }
}

fn generate_stress_test_config() -> SequencerConfig {
    // Create public public keypairs
    let public_keypairs = generate_public_keypairs();
    let initial_public_accounts = public_keypairs
        .iter()
        .map(|(_, addr)| AccountInitialData {
            addr: addr.to_string(),
            balance: 10000,
        })
        .collect();

    let sender_nsk = [1; 32];
    let sender_npk = NullifierPublicKey::from(&sender_nsk);
    let account = Account {
        balance: 100,
        nonce: 0xdeadbeef,
        program_owner: Program::authenticated_transfer_program().id(),
        data: vec![],
    };
    let initial_commitment = CommitmentsInitialData {
        npk: sender_npk,
        account,
    };

    SequencerConfig {
        home: ".".into(),
        override_rust_log: None,
        genesis_id: 1,
        is_genesis_random: true,
        max_num_tx_in_block: 20,
        block_create_timeout_millis: 10000,
        port: 3040,
        initial_accounts: initial_public_accounts,
        initial_commitments: vec![initial_commitment],
        signing_key: [37; 32],
    }
}

pub fn generate_public_keypairs() -> Vec<(PrivateKey, Address)> {
    const N_PUBLIC_ACCOUNTS_WITH_BALANCE: usize = 100000;

    (1..(N_PUBLIC_ACCOUNTS_WITH_BALANCE + 1))
        .map(|i| {
            let mut private_key_bytes = [0u8; 32];
            private_key_bytes[..8].copy_from_slice(&i.to_le_bytes());
            let private_key = PrivateKey::try_new(private_key_bytes).unwrap();
            let public_key = PublicKey::new_from_private_key(&private_key);
            let address = Address::from(&public_key);
            (private_key, address)
        })
        .collect::<Vec<_>>()
}

pub fn build_privacy_tx() -> PrivacyPreservingTransaction {
    let program = Program::authenticated_transfer_program();
    let sender_nsk = [1; 32];
    let sender_isk = [99; 32];
    let sender_ipk = IncomingViewingPublicKey::from_scalar(sender_isk);
    let sender_npk = NullifierPublicKey::from(&sender_nsk);
    let sender_pre = AccountWithMetadata::new(
        Account {
            balance: 100,
            nonce: 0xdeadbeef,
            program_owner: program.id(),
            data: vec![],
        },
        true,
        AccountId::from(&sender_npk),
    );
    let recipient_nsk = [2; 32];
    let recipient_isk = [99; 32];
    let recipient_ipk = IncomingViewingPublicKey::from_scalar(recipient_isk);
    let recipient_npk = NullifierPublicKey::from(&recipient_nsk);
    let recipient_pre =
        AccountWithMetadata::new(Account::default(), false, AccountId::from(&recipient_npk));
    let commitment_sender = Commitment::new(&sender_npk, &sender_pre.account);

    let eph_holder_from = EphemeralKeyHolder::new(&sender_npk);
    let sender_ss = eph_holder_from.calculate_shared_secret_sender(&sender_ipk);
    let sender_epk = eph_holder_from.generate_ephemeral_public_key();

    let eph_holder_to = EphemeralKeyHolder::new(&recipient_npk);
    let recipient_ss = eph_holder_to.calculate_shared_secret_sender(&recipient_ipk);
    let recipient_epk = eph_holder_from.generate_ephemeral_public_key();

    let balance_to_move: u128 = 1;
    let proof: MembershipProof = (
        1,
        vec![[
            170, 10, 217, 228, 20, 35, 189, 177, 238, 235, 97, 129, 132, 89, 96, 247, 86, 91, 222,
            214, 38, 194, 216, 67, 56, 251, 208, 226, 0, 117, 149, 39,
        ]],
    );
    let (output, proof) = circuit::execute_and_prove(
        &[sender_pre, recipient_pre],
        &Program::serialize_instruction(balance_to_move).unwrap(),
        &[1, 2],
        &[0xdeadbeef1, 0xdeadbeef2],
        &[
            (sender_npk.clone(), sender_ss),
            (recipient_npk.clone(), recipient_ss),
        ],
        &[(sender_nsk, proof)],
        &program,
    )
    .unwrap();
    let message = pptx::message::Message::try_from_circuit_output(
        vec![],
        vec![],
        vec![
            (sender_npk, sender_ipk, sender_epk),
            (recipient_npk, recipient_ipk, recipient_epk),
        ],
        output,
    )
    .unwrap();
    let witness_set = pptx::witness_set::WitnessSet::for_message(&message, proof, &[]);
    pptx::PrivacyPreservingTransaction::new(message, witness_set)
}

pub fn build_public_txs() -> Vec<PublicTransaction> {
    // Create public public keypairs
    let public_keypairs = generate_public_keypairs();

    // Create random private keychains
    // TODO

    // Create valid public transactions
    let program = Program::authenticated_transfer_program();
    let public_txs: Vec<PublicTransaction> = public_keypairs
        .windows(2)
        .map(|pair| {
            let amount: u128 = 1;
            let message = putx::Message::try_new(
                program.id(),
                [pair[0].1, pair[1].1].to_vec(),
                [0u128].to_vec(),
                amount,
            )
            .unwrap();
            let witness_set =
                nssa::public_transaction::WitnessSet::for_message(&message, &[&pair[0].0]);
            PublicTransaction::new(message, witness_set)
        })
        .collect();

    public_txs

    // Create valid private transactions
    // TODO
}
