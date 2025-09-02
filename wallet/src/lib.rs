use std::sync::Arc;

use common::{
    sequencer_client::{json::SendTxResponse, SequencerClient},
    ExecutionFailureKind,
};

use anyhow::Result;
use chain_storage::WalletChainStore;
use config::WalletConfig;
use log::info;
use nssa::Address;

use clap::{Parser, Subcommand};
use nssa_core::account::Account;

use crate::helperfunctions::{fetch_config, produce_account_addr_from_hex};

pub const HOME_DIR_ENV_VAR: &str = "NSSA_WALLET_HOME_DIR";
pub const BLOCK_GEN_DELAY_SECS: u64 = 20;

pub mod chain_storage;
pub mod config;
pub mod helperfunctions;

pub struct WalletCore {
    pub storage: WalletChainStore,
    pub sequencer_client: Arc<SequencerClient>,
}

impl WalletCore {
    pub fn start_from_config_update_chain(config: WalletConfig) -> Result<Self> {
        let client = Arc::new(SequencerClient::new(config.sequencer_addr.clone())?);

        let storage = WalletChainStore::new(config)?;

        Ok(Self {
            storage,
            sequencer_client: client.clone(),
        })
    }

    pub fn create_new_account(&mut self) -> Address {
        self.storage.user_data.generate_new_account()
    }

    pub fn search_for_initial_account(&self, acc_addr: Address) -> Option<Account> {
        for initial_acc in &self.storage.wallet_config.initial_accounts {
            if initial_acc.address == acc_addr {
                return Some(initial_acc.account.clone());
            }
        }
        None
    }

    pub async fn send_public_native_token_transfer(
        &self,
        from: Address,
        nonce: u128,
        to: Address,
        balance_to_move: u128,
    ) -> Result<SendTxResponse, ExecutionFailureKind> {
        let account = self.search_for_initial_account(from);

        if let Some(account) = account {
            if account.balance >= balance_to_move {
                let addresses = vec![from, to];
                let nonces = vec![nonce];
                let program_id = nssa::program::Program::authenticated_transfer_program().id();
                let message = nssa::public_transaction::Message::try_new(
                    program_id,
                    addresses,
                    nonces,
                    balance_to_move,
                )
                .unwrap();

                let signing_key = self.storage.user_data.get_account_signing_key(&from);

                if let Some(signing_key) = signing_key {
                    let witness_set =
                        nssa::public_transaction::WitnessSet::for_message(&message, &[signing_key]);

                    let tx = nssa::PublicTransaction::new(message, witness_set);

                    Ok(self.sequencer_client.send_tx(tx).await?)
                } else {
                    Err(ExecutionFailureKind::KeyNotFoundError)
                }
            } else {
                Err(ExecutionFailureKind::InsufficientFundsError)
            }
        } else {
            Err(ExecutionFailureKind::AmountMismatchError)
        }
    }
}

///Represents CLI command for a wallet
#[derive(Subcommand, Debug, Clone)]
#[clap(about)]
pub enum Command {
    ///Send native token transfer from `from` to `to` for `amount`
    SendNativeTokenTransfer {
        ///from - valid 32 byte hex string
        #[arg(long)]
        from: String,
        ///nonce - u128 integer
        #[arg(long)]
        nonce: u128,
        ///to - valid 32 byte hex string
        #[arg(long)]
        to: String,
        ///amount - amount of balance to move
        #[arg(long)]
        amount: u128,
    },
}

///To execute commands, env var NSSA_WALLET_HOME_DIR must be set into directory with config
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Wallet command
    #[command(subcommand)]
    pub command: Command,
}

pub async fn execute_subcommand(command: Command) -> Result<()> {
    match command {
        Command::SendNativeTokenTransfer {
            from,
            nonce,
            to,
            amount,
        } => {
            let wallet_config = fetch_config()?;

            let from = produce_account_addr_from_hex(from)?;
            let to = produce_account_addr_from_hex(to)?;

            let wallet_core = WalletCore::start_from_config_update_chain(wallet_config)?;

            let res = wallet_core
                .send_public_native_token_transfer(from, nonce, to, amount)
                .await?;

            info!("Results of tx send is {res:#?}");

            //ToDo: Insert transaction polling logic here
        }
    }

    Ok(())
}
