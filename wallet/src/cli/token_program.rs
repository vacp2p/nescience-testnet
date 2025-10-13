use anyhow::Result;
use clap::Subcommand;
use common::transaction::NSSATransaction;
use nssa::Address;

use crate::{SubcommandReturnValue, WalletCore, cli::WalletSubcommand};

///Represents CLI subcommand for a wallet working with token_program
#[derive(Subcommand, Debug, Clone)]
pub enum TokenProgramSubcommand {
    //Create a new token using the token program
    CreateNewToken {
        #[arg(short, long)]
        definition_addr: String,
        #[arg(short, long)]
        supply_addr: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        total_supply: u128,
    },
    //Transfer tokens using the token program
    TransferToken {
        #[arg(short, long)]
        sender_addr: String,
        #[arg(short, long)]
        recipient_addr: String,
        #[arg(short, long)]
        balance_to_move: u128,
    },
    //Create a new token using the token program
    CreateNewTokenPrivateOwned {
        #[arg(short, long)]
        definition_addr: String,
        #[arg(short, long)]
        supply_addr: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        total_supply: u128,
    },
    //Transfer tokens using the token program
    TransferTokenPrivateOwnedAlreadyInitialized {
        #[arg(short, long)]
        sender_addr: String,
        #[arg(short, long)]
        recipient_addr: String,
        #[arg(short, long)]
        balance_to_move: u128,
    },
    //Transfer tokens using the token program
    TransferTokenPrivateOwnedNotInitialized {
        #[arg(short, long)]
        sender_addr: String,
        #[arg(short, long)]
        recipient_addr: String,
        #[arg(short, long)]
        balance_to_move: u128,
    },
    //Transfer tokens using the token program
    TransferTokenPrivateForeign {
        #[arg(short, long)]
        sender_addr: String,
        ///recipient_npk - valid 32 byte hex string
        #[arg(long)]
        recipient_npk: String,
        ///recipient_ipk - valid 33 byte hex string
        #[arg(long)]
        recipient_ipk: String,
        #[arg(short, long)]
        balance_to_move: u128,
    },
}

impl WalletSubcommand for TokenProgramSubcommand {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            TokenProgramSubcommand::CreateNewToken {
                definition_addr,
                supply_addr,
                name,
                total_supply,
            } => {
                let name = name.as_bytes();
                if name.len() > 6 {
                    // TODO: return error
                    panic!();
                }
                let mut name_bytes = [0; 6];
                name_bytes[..name.len()].copy_from_slice(name);
                wallet_core
                    .send_new_token_definition(
                        definition_addr.parse().unwrap(),
                        supply_addr.parse().unwrap(),
                        name_bytes,
                        total_supply,
                    )
                    .await?;
                Ok(SubcommandReturnValue::Empty)
            }
            TokenProgramSubcommand::CreateNewTokenPrivateOwned {
                definition_addr,
                supply_addr,
                name,
                total_supply,
            } => {
                let name = name.as_bytes();
                if name.len() > 6 {
                    // TODO: return error
                    panic!("Name length mismatch");
                }
                let mut name_bytes = [0; 6];
                name_bytes[..name.len()].copy_from_slice(name);

                let definition_addr: Address = definition_addr.parse().unwrap();
                let supply_addr: Address = supply_addr.parse().unwrap();

                let (res, [secret_supply]) = wallet_core
                    .send_new_token_definition_private_owned(
                        definition_addr,
                        supply_addr,
                        name_bytes,
                        total_supply,
                    )
                    .await?;

                println!("Results of tx send is {res:#?}");

                let tx_hash = res.tx_hash;
                let transfer_tx = wallet_core
                    .poll_native_token_transfer(tx_hash.clone())
                    .await?;

                if let NSSATransaction::PrivacyPreserving(tx) = transfer_tx {
                    let supply_ebc = tx.message.encrypted_private_post_states[0].clone();
                    let supply_comm = tx.message.new_commitments[0].clone();

                    let res_acc_supply = nssa_core::EncryptionScheme::decrypt(
                        &supply_ebc.ciphertext,
                        &secret_supply,
                        &supply_comm,
                        0,
                    )
                    .unwrap();

                    println!("Received new to acc {res_acc_supply:#?}");

                    println!("Transaction data is {:?}", tx.message);

                    wallet_core
                        .storage
                        .insert_private_account_data(supply_addr, res_acc_supply);
                }

                let path = wallet_core.store_persistent_accounts()?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::PrivacyPreservingTransfer { tx_hash })
            }
            TokenProgramSubcommand::TransferToken {
                sender_addr,
                recipient_addr,
                balance_to_move,
            } => {
                wallet_core
                    .send_transfer_token_transaction(
                        sender_addr.parse().unwrap(),
                        recipient_addr.parse().unwrap(),
                        balance_to_move,
                    )
                    .await?;
                Ok(SubcommandReturnValue::Empty)
            }
            TokenProgramSubcommand::TransferTokenPrivateOwnedAlreadyInitialized {
                sender_addr,
                recipient_addr,
                balance_to_move,
            } => {
                let sender_addr: Address = sender_addr.parse().unwrap();
                let recipient_addr: Address = recipient_addr.parse().unwrap();

                let (res, [secret_sender, secret_recipient]) = wallet_core
                    .send_transfer_token_transaction_private_owned_account_already_initialized(
                        sender_addr,
                        recipient_addr,
                        balance_to_move,
                    )
                    .await?;

                println!("Results of tx send is {res:#?}");

                let tx_hash = res.tx_hash;
                let transfer_tx = wallet_core
                    .poll_native_token_transfer(tx_hash.clone())
                    .await?;

                if let NSSATransaction::PrivacyPreserving(tx) = transfer_tx {
                    let sender_ebc = tx.message.encrypted_private_post_states[0].clone();
                    let sender_comm = tx.message.new_commitments[0].clone();

                    let recipient_ebc = tx.message.encrypted_private_post_states[1].clone();
                    let recipient_comm = tx.message.new_commitments[1].clone();

                    let res_acc_sender = nssa_core::EncryptionScheme::decrypt(
                        &sender_ebc.ciphertext,
                        &secret_sender,
                        &sender_comm,
                        0,
                    )
                    .unwrap();

                    let res_acc_recipient = nssa_core::EncryptionScheme::decrypt(
                        &recipient_ebc.ciphertext,
                        &secret_recipient,
                        &recipient_comm,
                        1,
                    )
                    .unwrap();

                    println!("Received new sender acc {res_acc_sender:#?}");
                    println!("Received new recipient acc {res_acc_recipient:#?}");

                    println!("Transaction data is {:?}", tx.message);

                    wallet_core
                        .storage
                        .insert_private_account_data(sender_addr, res_acc_sender);
                    wallet_core
                        .storage
                        .insert_private_account_data(recipient_addr, res_acc_recipient);
                }

                let path = wallet_core.store_persistent_accounts()?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::PrivacyPreservingTransfer { tx_hash })
            }
            TokenProgramSubcommand::TransferTokenPrivateOwnedNotInitialized {
                sender_addr,
                recipient_addr,
                balance_to_move,
            } => {
                let sender_addr: Address = sender_addr.parse().unwrap();
                let recipient_addr: Address = recipient_addr.parse().unwrap();

                let (res, [secret_sender, secret_recipient]) = wallet_core
                    .send_transfer_token_transaction_private_owned_account_not_initialized(
                        sender_addr,
                        recipient_addr,
                        balance_to_move,
                    )
                    .await?;

                println!("Results of tx send is {res:#?}");

                let tx_hash = res.tx_hash;
                let transfer_tx = wallet_core
                    .poll_native_token_transfer(tx_hash.clone())
                    .await?;

                if let NSSATransaction::PrivacyPreserving(tx) = transfer_tx {
                    let sender_ebc = tx.message.encrypted_private_post_states[0].clone();
                    let sender_comm = tx.message.new_commitments[0].clone();

                    let recipient_ebc = tx.message.encrypted_private_post_states[1].clone();
                    let recipient_comm = tx.message.new_commitments[1].clone();

                    let res_acc_sender = nssa_core::EncryptionScheme::decrypt(
                        &sender_ebc.ciphertext,
                        &secret_sender,
                        &sender_comm,
                        0,
                    )
                    .unwrap();

                    let res_acc_recipient = nssa_core::EncryptionScheme::decrypt(
                        &recipient_ebc.ciphertext,
                        &secret_recipient,
                        &recipient_comm,
                        1,
                    )
                    .unwrap();

                    println!("Received new sender acc {res_acc_sender:#?}");
                    println!("Received new recipient acc {res_acc_recipient:#?}");

                    println!("Transaction data is {:?}", tx.message);

                    wallet_core
                        .storage
                        .insert_private_account_data(sender_addr, res_acc_sender);
                    wallet_core
                        .storage
                        .insert_private_account_data(recipient_addr, res_acc_recipient);
                }

                let path = wallet_core.store_persistent_accounts()?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::PrivacyPreservingTransfer { tx_hash })
            }
            TokenProgramSubcommand::TransferTokenPrivateForeign {
                sender_addr,
                recipient_npk,
                recipient_ipk,
                balance_to_move,
            } => {
                let sender_addr: Address = sender_addr.parse().unwrap();
                let recipient_npk_res = hex::decode(recipient_npk)?;
                let mut recipient_npk = [0; 32];
                recipient_npk.copy_from_slice(&recipient_npk_res);
                let recipient_npk = nssa_core::NullifierPublicKey(recipient_npk);

                let recipient_ipk_res = hex::decode(recipient_ipk)?;
                let mut recipient_ipk = [0u8; 33];
                recipient_ipk.copy_from_slice(&recipient_ipk_res);
                let recipient_ipk = nssa_core::encryption::shared_key_derivation::Secp256k1Point(
                    recipient_ipk.to_vec(),
                );

                let (res, [secret_sender, _]) = wallet_core
                    .send_transfer_token_transaction_private_foreign_account(
                        sender_addr,
                        recipient_npk,
                        recipient_ipk,
                        balance_to_move,
                    )
                    .await?;

                println!("Results of tx send is {res:#?}");

                let tx_hash = res.tx_hash;
                let transfer_tx = wallet_core
                    .poll_native_token_transfer(tx_hash.clone())
                    .await?;

                if let NSSATransaction::PrivacyPreserving(tx) = transfer_tx {
                    let sender_ebc = tx.message.encrypted_private_post_states[0].clone();
                    let sender_comm = tx.message.new_commitments[0].clone();

                    let res_acc_sender = nssa_core::EncryptionScheme::decrypt(
                        &sender_ebc.ciphertext,
                        &secret_sender,
                        &sender_comm,
                        0,
                    )
                    .unwrap();

                    println!("Received new sender acc {res_acc_sender:#?}");

                    println!("Transaction data is {:?}", tx.message);

                    wallet_core
                        .storage
                        .insert_private_account_data(sender_addr, res_acc_sender);
                }

                let path = wallet_core.store_persistent_accounts()?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::PrivacyPreservingTransfer { tx_hash })
            }
        }
    }
}
