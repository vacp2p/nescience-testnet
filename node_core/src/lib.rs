use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use accounts::account_core::Account;
use anyhow::Result;
use config::NodeConfig;
use sequencer_client::SequencerClient;
use storage::NodeChainStore;
use tokio::{sync::Mutex, task::JoinHandle};

pub mod config;
pub mod executions;
pub mod sequencer_client;
pub mod storage;

pub struct NodeCore {
    pub storage: Arc<Mutex<NodeChainStore>>,
    pub curr_height: Arc<AtomicU64>,
    pub main_acc: Account,
    pub node_config: NodeConfig,
    pub db_updater_handle: JoinHandle<Result<()>>,
}

impl NodeCore {
    pub async fn start_from_config_update_chain(config: NodeConfig) -> Result<Self> {
        let client = SequencerClient::new(config.clone())?;

        let genesis_id = client.get_genesis_id().await?;
        let genesis_block = client.get_block(genesis_id.genesis_id).await?.block;

        let mut storage = NodeChainStore::new_with_genesis(&config.home, genesis_block);

        let account = Account::new();

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
            main_acc: account,
            node_config: config.clone(),
            db_updater_handle: updater_handle,
        })
    }
}
