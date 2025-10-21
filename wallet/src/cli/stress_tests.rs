use std::fs::File;

use anyhow::Result;
use clap::Subcommand;
use nssa::{
    Address, PrivateKey, PublicKey, PublicTransaction, program::Program,
    public_transaction::Message,
};
use sequencer_core::config::{AccountInitialData, SequencerConfig};

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
                let txs = build_txs();
                for (i, tx) in txs.into_iter().enumerate() {
                    wallet_core.sequencer_client.send_tx_public(tx).await;
                    println!("Sent tx: {}", i);
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

    SequencerConfig {
        home: ".".into(),
        override_rust_log: None,
        genesis_id: 1,
        is_genesis_random: true,
        max_num_tx_in_block: 20,
        block_create_timeout_millis: 10000,
        port: 3040,
        initial_accounts: initial_public_accounts,
        initial_commitments: vec![],
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

pub fn build_txs() -> Vec<PublicTransaction> {
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
            let message = Message::try_new(
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
