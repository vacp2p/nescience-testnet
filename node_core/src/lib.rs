use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use ::storage::transaction::{Transaction, TransactionPayload, TxKind};
use accounts::account_core::{Account, AccountAddress};
use anyhow::Result;
use config::NodeConfig;
use executions::{
    private_exec::{generate_commitments, generate_nullifiers},
    se::{commit, tag_random},
};
use rand::thread_rng;
use secp256k1_zkp::{CommitmentSecrets, Tweak};
use sequencer_client::{json::SendTxResponse, SequencerClient};
use serde::{Deserialize, Serialize};
use storage::NodeChainStore;
use tokio::{sync::Mutex, task::JoinHandle};
use utxo::utxo_core::UTXO;
use zkvm::{
    prove_mint_utxo, prove_send_utxo, prove_send_utxo_deshielded, prove_send_utxo_shielded,
};

pub mod config;
pub mod executions;
pub mod sequencer_client;
pub mod storage;

#[derive(Debug, Serialize, Deserialize)]
pub struct MintMoneyPublicTx {
    pub acc: AccountAddress,
    pub amount: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMoneyShieldedTx {
    pub acc_sender: AccountAddress,
    pub amount: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMoneyDeshieldedTx {
    pub receiver_data: Vec<(u128, AccountAddress)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActionData {
    MintMoneyPublicTx(MintMoneyPublicTx),
    SendMoneyShieldedTx(SendMoneyShieldedTx),
    SendMoneyDeshieldedTx(SendMoneyDeshieldedTx),
}

pub struct NodeCore {
    pub storage: Arc<Mutex<NodeChainStore>>,
    pub curr_height: Arc<AtomicU64>,
    pub acc_map: HashMap<AccountAddress, Account>,
    pub node_config: NodeConfig,
    pub db_updater_handle: JoinHandle<Result<()>>,
    pub sequencer_client: Arc<SequencerClient>,
}

impl NodeCore {
    pub async fn start_from_config_update_chain(config: NodeConfig) -> Result<Self> {
        let client = Arc::new(SequencerClient::new(config.clone())?);

        let genesis_id = client.get_genesis_id().await?;
        let genesis_block = client.get_block(genesis_id.genesis_id).await?.block;

        let mut storage = NodeChainStore::new_with_genesis(&config.home, genesis_block);

        let account_map = HashMap::new();

        let mut chain_height = genesis_id.genesis_id;

        //Chain update loop
        loop {
            let next_block = chain_height + 1;

            if let Ok(block) = client.get_block(next_block).await {
                storage.dissect_insert_block(block.block)?;
            } else {
                break;
            }

            chain_height += 1;
        }

        let wrapped_storage = Arc::new(Mutex::new(storage));
        let chain_height_wrapped = Arc::new(AtomicU64::new(chain_height));

        let wrapped_storage_thread = wrapped_storage.clone();
        let wrapped_chain_height_thread = chain_height_wrapped.clone();
        let client_thread = client.clone();

        let updater_handle = tokio::spawn(async move {
            loop {
                let next_block = wrapped_chain_height_thread.load(Ordering::Relaxed) + 1;

                if let Ok(block) = client_thread.get_block(next_block).await {
                    {
                        let mut storage_guard = wrapped_storage_thread.lock().await;

                        storage_guard.dissect_insert_block(block.block)?;
                    }

                    wrapped_chain_height_thread.store(next_block, Ordering::Relaxed);
                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(
                        config.seq_poll_timeout_secs,
                    ))
                    .await;
                }
            }
        });

        Ok(Self {
            storage: wrapped_storage,
            curr_height: chain_height_wrapped,
            acc_map: account_map,
            node_config: config.clone(),
            db_updater_handle: updater_handle,
            sequencer_client: client.clone(),
        })
    }

    pub fn mint_utxo_private(&self, acc: AccountAddress, amount: u128) -> Transaction {
        let (utxo, receipt) = prove_mint_utxo(amount, acc);

        let accout = self.acc_map.get(&acc).unwrap();

        let encoded_data = Account::encrypt_data(
            &accout.produce_ephemeral_key_holder(),
            accout.key_holder.viewing_public_key,
            &serde_json::to_vec(&utxo).unwrap(),
        );

        let comm = generate_commitments(&vec![utxo]);

        TransactionPayload {
            tx_kind: TxKind::Private,
            execution_input: vec![],
            execution_output: vec![],
            utxo_commitments_spent_hashes: vec![],
            utxo_commitments_created_hashes: comm
                .into_iter()
                .map(|hash_data| hash_data.try_into().unwrap())
                .collect(),
            nullifier_created_hashes: vec![],
            execution_proof_private: serde_json::to_string(&receipt).unwrap(),
            encoded_data: vec![(encoded_data.0, encoded_data.1.to_vec())],
        }
        .into()
    }

    pub fn deposit_money_public(&self, acc: AccountAddress, amount: u128) -> Transaction {
        TransactionPayload {
            tx_kind: TxKind::Public,
            execution_input: serde_json::to_vec(&ActionData::MintMoneyPublicTx(
                MintMoneyPublicTx { acc, amount },
            ))
            .unwrap(),
            execution_output: vec![],
            utxo_commitments_spent_hashes: vec![],
            utxo_commitments_created_hashes: vec![],
            nullifier_created_hashes: vec![],
            execution_proof_private: "".to_string(),
            encoded_data: vec![],
        }
        .into()
    }

    pub async fn transfer_utxo_private(
        &self,
        utxo: UTXO,
        receivers: Vec<(u128, AccountAddress)>,
    ) -> Transaction {
        let accout = self.acc_map.get(&utxo.owner).unwrap();

        let commitment_in = {
            let guard = self.storage.lock().await;

            guard.utxo_commitments_store.get_tx(utxo.hash).unwrap().hash
        };

        let nullifier = generate_nullifiers(
            &utxo,
            &accout
                .key_holder
                .utxo_secret_key_holder
                .nullifier_secret_key
                .to_bytes()
                .to_vec(),
        );

        let (resulting_utxos, receipt) = prove_send_utxo(utxo, receivers);

        let utxos: Vec<UTXO> = resulting_utxos
            .iter()
            .map(|(utxo, _)| utxo.clone())
            .collect();

        let encoded_data: Vec<(Vec<u8>, Vec<u8>)> = utxos
            .iter()
            .map(|utxo_enc| {
                let accout_enc = self.acc_map.get(&utxo_enc.owner).unwrap();

                let (ciphertext, nonce) = Account::encrypt_data(
                    &accout.produce_ephemeral_key_holder(),
                    accout_enc.key_holder.viewing_public_key,
                    &serde_json::to_vec(&utxo_enc).unwrap(),
                );

                (ciphertext, nonce.to_vec())
            })
            .collect();

        let commitments = generate_commitments(&utxos);

        TransactionPayload {
            tx_kind: TxKind::Private,
            execution_input: vec![],
            execution_output: vec![],
            utxo_commitments_spent_hashes: vec![commitment_in],
            utxo_commitments_created_hashes: commitments
                .into_iter()
                .map(|hash_data| hash_data.try_into().unwrap())
                .collect(),
            nullifier_created_hashes: vec![nullifier.try_into().unwrap()],
            execution_proof_private: serde_json::to_string(&receipt).unwrap(),
            encoded_data,
        }
        .into()
    }

    pub async fn transfer_balance_shielded(
        &self,
        acc: AccountAddress,
        balance: u64,
        receivers: Vec<(u128, AccountAddress)>,
    ) -> Transaction {
        let accout = self.acc_map.get(&acc).unwrap();

        let commitment_secrets = CommitmentSecrets {
            value: balance,
            value_blinding_factor: Tweak::from_slice(
                &accout
                    .key_holder
                    .utxo_secret_key_holder
                    .viewing_secret_key
                    .to_bytes()
                    .to_vec(),
            )
            .unwrap(),
            generator_blinding_factor: Tweak::new(&mut thread_rng()),
        };

        let tag = tag_random();
        let commitment = commit(&commitment_secrets, tag);

        let nullifier = executions::se::generate_nullifiers(
            &commitment,
            &accout
                .key_holder
                .utxo_secret_key_holder
                .nullifier_secret_key
                .to_bytes()
                .to_vec(),
        );

        let (resulting_utxos, receipt) = prove_send_utxo_shielded(acc, balance as u128, receivers);

        let utxos: Vec<UTXO> = resulting_utxos
            .iter()
            .map(|(utxo, _)| utxo.clone())
            .collect();

        let encoded_data: Vec<(Vec<u8>, Vec<u8>)> = utxos
            .iter()
            .map(|utxo_enc| {
                let accout_enc = self.acc_map.get(&utxo_enc.owner).unwrap();

                let (ciphertext, nonce) = Account::encrypt_data(
                    &accout.produce_ephemeral_key_holder(),
                    accout_enc.key_holder.viewing_public_key,
                    &serde_json::to_vec(&utxo_enc).unwrap(),
                );

                (ciphertext, nonce.to_vec())
            })
            .collect();

        let commitments = generate_commitments(&utxos);

        TransactionPayload {
            tx_kind: TxKind::Private,
            execution_input: serde_json::to_vec(&ActionData::SendMoneyShieldedTx(
                SendMoneyShieldedTx {
                    acc_sender: acc,
                    amount: balance as u128,
                },
            ))
            .unwrap(),
            execution_output: vec![],
            utxo_commitments_spent_hashes: vec![],
            utxo_commitments_created_hashes: commitments
                .into_iter()
                .map(|hash_data| hash_data.try_into().unwrap())
                .collect(),
            nullifier_created_hashes: vec![nullifier.try_into().unwrap()],
            execution_proof_private: serde_json::to_string(&receipt).unwrap(),
            encoded_data,
        }
        .into()
    }

    pub async fn transfer_utxo_deshielded(
        &self,
        utxo: UTXO,
        receivers: Vec<(u128, AccountAddress)>,
    ) -> Transaction {
        let accout = self.acc_map.get(&utxo.owner).unwrap();

        let commitment_in = {
            let guard = self.storage.lock().await;

            guard.utxo_commitments_store.get_tx(utxo.hash).unwrap().hash
        };

        let nullifier = generate_nullifiers(
            &utxo,
            &accout
                .key_holder
                .utxo_secret_key_holder
                .nullifier_secret_key
                .to_bytes()
                .to_vec(),
        );

        let (resulting_utxos, receipt) = prove_send_utxo_deshielded(utxo, receivers);

        TransactionPayload {
            tx_kind: TxKind::Private,
            execution_input: vec![],
            execution_output: serde_json::to_vec(&ActionData::SendMoneyDeshieldedTx(
                SendMoneyDeshieldedTx {
                    receiver_data: resulting_utxos,
                },
            ))
            .unwrap(),
            utxo_commitments_spent_hashes: vec![commitment_in],
            utxo_commitments_created_hashes: vec![],
            nullifier_created_hashes: vec![nullifier.try_into().unwrap()],
            execution_proof_private: serde_json::to_string(&receipt).unwrap(),
            encoded_data: vec![],
        }
        .into()
    }

    pub async fn send_private_mint_tx(
        &self,
        acc: AccountAddress,
        amount: u128,
    ) -> Result<SendTxResponse> {
        Ok(self
            .sequencer_client
            .send_tx(self.mint_utxo_private(acc, amount))
            .await?)
    }

    pub async fn send_public_deposit(
        &self,
        acc: AccountAddress,
        amount: u128,
    ) -> Result<SendTxResponse> {
        Ok(self
            .sequencer_client
            .send_tx(self.deposit_money_public(acc, amount))
            .await?)
    }

    pub async fn send_private_send_tx(
        &self,
        utxo: UTXO,
        receivers: Vec<(u128, AccountAddress)>,
    ) -> Result<SendTxResponse> {
        Ok(self
            .sequencer_client
            .send_tx(self.transfer_utxo_private(utxo, receivers).await)
            .await?)
    }

    pub async fn send_shielded_send_tx(
        &self,
        acc: AccountAddress,
        amount: u64,
        receivers: Vec<(u128, AccountAddress)>,
    ) -> Result<SendTxResponse> {
        Ok(self
            .sequencer_client
            .send_tx(self.transfer_balance_shielded(acc, amount, receivers).await)
            .await?)
    }

    pub async fn send_deshielded_send_tx(
        &self,
        utxo: UTXO,
        receivers: Vec<(u128, AccountAddress)>,
    ) -> Result<SendTxResponse> {
        Ok(self
            .sequencer_client
            .send_tx(self.transfer_utxo_deshielded(utxo, receivers).await)
            .await?)
    }
}
