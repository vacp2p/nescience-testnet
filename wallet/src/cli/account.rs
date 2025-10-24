use anyhow::Result;
use base58::ToBase58;
use clap::Subcommand;
use common::transaction::NSSATransaction;
use nssa::{Address, program::Program};
use serde::Serialize;

use crate::{
    SubcommandReturnValue, WalletCore,
    cli::WalletSubcommand,
    helperfunctions::{AddressPrivacyKind, HumanReadableAccount, parse_addr_with_privacy_prefix},
};

const TOKEN_DEFINITION_TYPE: u8 = 0;
const TOKEN_DEFINITION_DATA_SIZE: usize = 23;

const TOKEN_HOLDING_TYPE: u8 = 1;
const TOKEN_HOLDING_DATA_SIZE: usize = 49;

struct TokenDefinition {
    #[allow(unused)]
    account_type: u8,
    name: [u8; 6],
    total_supply: u128,
}

struct TokenHolding {
    #[allow(unused)]
    account_type: u8,
    definition_id: Address,
    balance: u128,
}

impl TokenDefinition {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() != TOKEN_DEFINITION_DATA_SIZE || data[0] != TOKEN_DEFINITION_TYPE {
            None
        } else {
            let account_type = data[0];
            let name = data[1..7].try_into().unwrap();
            let total_supply = u128::from_le_bytes(data[7..].try_into().unwrap());

            Some(Self {
                account_type,
                name,
                total_supply,
            })
        }
    }
}

impl TokenHolding {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() != TOKEN_HOLDING_DATA_SIZE || data[0] != TOKEN_HOLDING_TYPE {
            None
        } else {
            let account_type = data[0];
            let definition_id = Address::new(data[1..33].try_into().unwrap());
            let balance = u128::from_le_bytes(data[33..].try_into().unwrap());
            Some(Self {
                definition_id,
                balance,
                account_type,
            })
        }
    }
}

///Represents generic chain CLI subcommand
#[derive(Subcommand, Debug, Clone)]
pub enum AccountSubcommand {
    ///Get
    Get {
        #[arg(long)]
        raw: bool,
        #[arg(short, long)]
        addr: String,
    },
    ///Fetch
    #[command(subcommand)]
    Fetch(FetchSubcommand),
    ///New
    #[command(subcommand)]
    New(NewSubcommand),
}

///Represents generic getter CLI subcommand
#[derive(Subcommand, Debug, Clone)]
pub enum FetchSubcommand {
    ///Fetch transaction by `hash`
    Tx {
        #[arg(short, long)]
        tx_hash: String,
    },
    ///Claim account `acc_addr` generated in transaction `tx_hash`, using secret `sh_secret` at ciphertext id `ciph_id`
    PrivateAccount {
        ///tx_hash - valid 32 byte hex string
        #[arg(long)]
        tx_hash: String,
        ///acc_addr - valid 32 byte hex string
        #[arg(long)]
        acc_addr: String,
        ///output_id - id of the output in the transaction
        #[arg(long)]
        output_id: usize,
    },
}

///Represents generic register CLI subcommand
#[derive(Subcommand, Debug, Clone)]
pub enum NewSubcommand {
    ///Register new public account
    Public {},
    ///Register new private account
    Private {},
}

impl WalletSubcommand for FetchSubcommand {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            FetchSubcommand::Tx { tx_hash } => {
                let tx_obj = wallet_core
                    .sequencer_client
                    .get_transaction_by_hash(tx_hash)
                    .await?;

                println!("Transaction object {tx_obj:#?}");

                Ok(SubcommandReturnValue::Empty)
            }
            FetchSubcommand::PrivateAccount {
                tx_hash,
                acc_addr,
                output_id: ciph_id,
            } => {
                let acc_addr: Address = acc_addr.parse().unwrap();

                let account_key_chain = wallet_core
                    .storage
                    .user_data
                    .user_private_accounts
                    .get(&acc_addr);

                let Some((account_key_chain, _)) = account_key_chain else {
                    anyhow::bail!("Account not found");
                };

                let transfer_tx = wallet_core.poll_native_token_transfer(tx_hash).await?;

                if let NSSATransaction::PrivacyPreserving(tx) = transfer_tx {
                    let to_ebc = tx.message.encrypted_private_post_states[ciph_id].clone();
                    let to_comm = tx.message.new_commitments[ciph_id].clone();
                    let shared_secret =
                        account_key_chain.calculate_shared_secret_receiver(to_ebc.epk);

                    let res_acc_to = nssa_core::EncryptionScheme::decrypt(
                        &to_ebc.ciphertext,
                        &shared_secret,
                        &to_comm,
                        ciph_id as u32,
                    )
                    .unwrap();

                    println!("RES acc to {res_acc_to:#?}");

                    println!("Transaction data is {:?}", tx.message);

                    wallet_core
                        .storage
                        .insert_private_account_data(acc_addr, res_acc_to);
                }

                let path = wallet_core.store_persistent_accounts().await?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::Empty)
            }
        }
    }
}

impl WalletSubcommand for NewSubcommand {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            NewSubcommand::Public {} => {
                let addr = wallet_core.create_new_account_public();

                println!("Generated new account with addr {addr}");

                let path = wallet_core.store_persistent_accounts().await?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::RegisterAccount { addr })
            }
            NewSubcommand::Private {} => {
                let addr = wallet_core.create_new_account_private();

                let (key, _) = wallet_core
                    .storage
                    .user_data
                    .get_private_account(&addr)
                    .unwrap();

                println!(
                    "Generated new account with addr {}",
                    addr.to_bytes().to_base58()
                );
                println!("With npk {}", hex::encode(&key.nullifer_public_key.0));
                println!(
                    "With ipk {}",
                    hex::encode(key.incoming_viewing_public_key.to_bytes())
                );

                let path = wallet_core.store_persistent_accounts().await?;

                println!("Stored persistent accounts at {path:#?}");

                Ok(SubcommandReturnValue::RegisterAccount { addr })
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthenticatedTransferAccountView {
    pub balance: u128,
}

impl From<nssa::Account> for AuthenticatedTransferAccountView {
    fn from(value: nssa::Account) -> Self {
        Self {
            balance: value.balance,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TokedDefinitionAccountView {
    pub account_type: String,
    pub name: String,
    pub total_supply: u128,
}

impl From<TokenDefinition> for TokedDefinitionAccountView {
    fn from(value: TokenDefinition) -> Self {
        Self {
            account_type: "Token definition".to_string(),
            name: hex::encode(value.name),
            total_supply: value.total_supply,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TokedHoldingAccountView {
    pub account_type: String,
    pub definition_id: String,
    pub balance: u128,
}

impl From<TokenHolding> for TokedHoldingAccountView {
    fn from(value: TokenHolding) -> Self {
        Self {
            account_type: "Token holding".to_string(),
            definition_id: value.definition_id.to_string(),
            balance: value.balance,
        }
    }
}

impl WalletSubcommand for AccountSubcommand {
    async fn handle_subcommand(
        self,
        wallet_core: &mut WalletCore,
    ) -> Result<SubcommandReturnValue> {
        match self {
            AccountSubcommand::Get { raw, addr } => {
                let (addr, addr_kind) = parse_addr_with_privacy_prefix(&addr)?;

                let account = match addr_kind {
                    AddressPrivacyKind::Public => wallet_core.get_account_public(addr).await?,
                    AddressPrivacyKind::Private => wallet_core
                        .get_account_private(&addr)
                        .ok_or(anyhow::anyhow!("Private account not found in storage"))?,
                };

                if raw {
                    let account_hr: HumanReadableAccount = account.clone().into();
                    println!("{}", serde_json::to_string(&account_hr).unwrap());

                    return Ok(SubcommandReturnValue::Empty);
                }

                let auth_tr_prog_id = Program::authenticated_transfer_program().id();
                let token_prog_id = Program::token().id();

                let acc_view = match &account.program_owner {
                    _ if &account.program_owner == &auth_tr_prog_id => {
                        let acc_view: AuthenticatedTransferAccountView = account.into();

                        serde_json::to_string(&acc_view)?
                    }
                    _ if &account.program_owner == &token_prog_id => {
                        if let Some(token_def) = TokenDefinition::parse(&account.data) {
                            let acc_view: TokedDefinitionAccountView = token_def.into();

                            serde_json::to_string(&acc_view)?
                        } else if let Some(token_hold) = TokenHolding::parse(&account.data) {
                            let acc_view: TokedHoldingAccountView = token_hold.into();

                            serde_json::to_string(&acc_view)?
                        } else {
                            anyhow::bail!("Invalid data for account {addr:#?} with token program");
                        }
                    }
                    _ => {
                        let account_hr: HumanReadableAccount = account.clone().into();
                        serde_json::to_string(&account_hr).unwrap()
                    }
                };

                println!("{}", acc_view);

                Ok(SubcommandReturnValue::Empty)
            }
            AccountSubcommand::Fetch(fetch_subcommand) => {
                fetch_subcommand.handle_subcommand(wallet_core).await
            }
            AccountSubcommand::New(new_subcommand) => {
                new_subcommand.handle_subcommand(wallet_core).await
            }
        }
    }
}
