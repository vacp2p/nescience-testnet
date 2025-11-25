use anyhow::Result;
use clap::Subcommand;
use common::{PINATA_BASE58, transaction::NSSATransaction};
use log::info;

use crate::{
    SubcommandReturnValue, WalletCore,
    cli::WalletSubcommand,
    helperfunctions::{AccountPrivacyKind, parse_addr_with_privacy_prefix},
};

///Represents generic CLI subcommand for a wallet working with pinata program
#[derive(Subcommand, Debug, Clone)]
pub enum PinataProgramAgnosticSubcommand {
    ///Claim pinata
    Claim {
        ///to_account_id - valid 32 byte base58 string with privacy prefix
        #[arg(long)]
        to_account_id: String,
        ///solution - solution to pinata challenge
        #[arg(long)]
        solution: u128,
    },
}

impl WalletSubcommand for PinataProgramAgnosticSubcommand {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        let underlying_subcommand = match self {
            PinataProgramAgnosticSubcommand::Claim {
                to_account_id,
                solution,
            } => {
                let (to_account_id, to_addr_privacy) =
                    parse_addr_with_privacy_prefix(&to_account_id)?;

                match to_addr_privacy {
                    AccountPrivacyKind::Public => {
                        PinataProgramSubcommand::Public(PinataProgramSubcommandPublic::Claim {
                            pinata_account_id: PINATA_BASE58.to_string(),
                            winner_account_id: to_account_id,
                            solution,
                        })
                    }
                    AccountPrivacyKind::Private => PinataProgramSubcommand::Private(
                        PinataProgramSubcommandPrivate::ClaimPrivateOwned {
                            pinata_account_id: PINATA_BASE58.to_string(),
                            winner_account_id: to_account_id,
                            solution,
                        },
                    ),
                }
            }
        };

        underlying_subcommand.handle_subcommand(wallet_core).await
    }
}

///Represents generic CLI subcommand for a wallet working with pinata program
#[derive(Subcommand, Debug, Clone)]
pub enum PinataProgramSubcommand {
    ///Public execution
    #[command(subcommand)]
    Public(PinataProgramSubcommandPublic),
    ///Private execution
    #[command(subcommand)]
    Private(PinataProgramSubcommandPrivate),
}

///Represents generic public CLI subcommand for a wallet working with pinata program
#[derive(Subcommand, Debug, Clone)]
pub enum PinataProgramSubcommandPublic {
    // TODO: Testnet only. Refactor to prevent compilation on mainnet.
    // Claim piñata prize
    Claim {
        ///pinata_account_id - valid 32 byte hex string
        #[arg(long)]
        pinata_account_id: String,
        ///winner_account_id - valid 32 byte hex string
        #[arg(long)]
        winner_account_id: String,
        ///solution - solution to pinata challenge
        #[arg(long)]
        solution: u128,
    },
}

///Represents generic private CLI subcommand for a wallet working with pinata program
#[derive(Subcommand, Debug, Clone)]
pub enum PinataProgramSubcommandPrivate {
    // TODO: Testnet only. Refactor to prevent compilation on mainnet.
    // Claim piñata prize
    ClaimPrivateOwned {
        ///pinata_account_id - valid 32 byte hex string
        #[arg(long)]
        pinata_account_id: String,
        ///winner_account_id - valid 32 byte hex string
        #[arg(long)]
        winner_account_id: String,
        ///solution - solution to pinata challenge
        #[arg(long)]
        solution: u128,
    },
}

impl WalletSubcommand for PinataProgramSubcommandPublic {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            PinataProgramSubcommandPublic::Claim {
                pinata_account_id,
                winner_account_id,
                solution,
            } => {
                let res = wallet_core
                    .claim_pinata(
                        pinata_account_id.parse().unwrap(),
                        winner_account_id.parse().unwrap(),
                        solution,
                    )
                    .await?;
                info!("Results of tx send is {res:#?}");

                Ok(SubcommandReturnValue::Empty)
            }
        }
    }
}

impl WalletSubcommand for PinataProgramSubcommandPrivate {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            PinataProgramSubcommandPrivate::ClaimPrivateOwned {
                pinata_account_id,
                winner_account_id,
                solution,
            } => {
                let pinata_account_id = pinata_account_id.parse().unwrap();
                let winner_account_id = winner_account_id.parse().unwrap();

                let winner_initialization = wallet_core
                    .check_private_account_initialized(&winner_account_id)
                    .await?;

                let (res, [secret_winner]) = if let Some(winner_proof) = winner_initialization {
                    wallet_core
                        .claim_pinata_private_owned_account_already_initialized(
                            pinata_account_id,
                            winner_account_id,
                            solution,
                            winner_proof,
                        )
                        .await?
                } else {
                    wallet_core
                        .claim_pinata_private_owned_account_not_initialized(
                            pinata_account_id,
                            winner_account_id,
                            solution,
                        )
                        .await?
                };

                info!("Results of tx send is {res:#?}");

                let tx_hash = res.tx_hash;
                let transfer_tx = wallet_core
                    .poll_native_token_transfer(tx_hash.clone())
                    .await?;

                if let NSSATransaction::PrivacyPreserving(tx) = transfer_tx {
                    let acc_decode_data = vec![(secret_winner, winner_account_id)];

                    wallet_core.decode_insert_privacy_preserving_transaction_results(
                        tx,
                        &acc_decode_data,
                    )?;
                }

                let path = wallet_core.store_persistent_data().await?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::PrivacyPreservingTransfer { tx_hash })
            }
        }
    }
}

impl WalletSubcommand for PinataProgramSubcommand {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            PinataProgramSubcommand::Private(private_subcommand) => {
                private_subcommand.handle_subcommand(wallet_core).await
            }
            PinataProgramSubcommand::Public(public_subcommand) => {
                public_subcommand.handle_subcommand(wallet_core).await
            }
        }
    }
}
