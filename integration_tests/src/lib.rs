use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use common::rpc_primitives::RpcConfig;
use log::info;
use node_core::NodeCore;
use sequencer_core::SequencerCore;
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Path to configs
    home_dir: PathBuf,
}

pub const ACC_SENDER: &str = "0d96dfcc414019380c9dde0cd3dce5aac90fb5443bf871108741aeafde552ad7";
pub const ACC_RECEIVER: &str = "974870e9be8d0ac08aa83b3fc7a7a686291d8732508aba98b36080f39c2cf364";

pub async fn main_tests_runner() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let Args { home_dir } = args;

    let home_dir_sequencer = home_dir.join("sequencer");
    let home_dir_node = home_dir.join("node");

    let sequencer_config =
        sequencer_runner::config::from_file(home_dir_sequencer.join("sequencer_config.json"))
            .unwrap();
    let node_config =
        node_runner::config::from_file(home_dir_node.join("node_config.json")).unwrap();

    let block_timeout = sequencer_config.block_create_timeout_millis;
    let sequencer_port = sequencer_config.port;

    let sequencer_core = SequencerCore::start_from_config(sequencer_config);

    info!("Sequencer core set up");

    let seq_core_wrapped = Arc::new(Mutex::new(sequencer_core));

    let http_server = sequencer_rpc::new_http_server(
        RpcConfig::with_port(sequencer_port),
        seq_core_wrapped.clone(),
    )?;
    info!("HTTP server started");
    let seq_http_server_handle = http_server.handle();
    tokio::spawn(http_server);

    info!("Starting main sequencer loop");

    let sequencer_loop_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(block_timeout)).await;

            info!("Collecting transactions from mempool, block creation");

            let id = {
                let mut state = seq_core_wrapped.lock().await;

                state.produce_new_block_with_mempool_transactions()?
            };

            info!("Block with id {id} created");

            info!("Waiting for new transactions");
        }
    });

    let node_port = node_config.port;

    let node_core = NodeCore::start_from_config_update_chain(node_config.clone()).await?;
    let wrapped_node_core = Arc::new(Mutex::new(node_core));

    let http_server = node_rpc::new_http_server(
        RpcConfig::with_port(node_port),
        node_config.clone(),
        wrapped_node_core.clone(),
    )?;
    info!("HTTP server started");
    let node_http_server_handle = http_server.handle();
    tokio::spawn(http_server);

    info!("Waiting for first block creation");
    tokio::time::sleep(Duration::from_secs(12)).await;

    let acc_sender = hex::decode(ACC_SENDER).unwrap().try_into().unwrap();
    let acc_receiver = hex::decode(ACC_RECEIVER).unwrap().try_into().unwrap();

    {
        let guard = wrapped_node_core.lock().await;

        let res = guard
            .send_public_native_token_transfer(acc_sender, acc_receiver, 100)
            .await
            .unwrap();

        info!("Res of tx_send is {res:#?}");

        info!("Waiting for next block creation");
        tokio::time::sleep(Duration::from_secs(12)).await;

        info!("Checking correct balance move");
        let acc_1_balance = guard
            .sequencer_client
            .get_account_balance(ACC_SENDER.to_string())
            .await
            .unwrap();
        let acc_2_balance = guard
            .sequencer_client
            .get_account_balance(ACC_RECEIVER.to_string())
            .await
            .unwrap();

        info!("Balance of sender : {acc_1_balance:#?}");
        info!("Balance of receiver : {acc_2_balance:#?}");
    }

    info!("Success!");

    info!("Cleanup");

    node_http_server_handle.stop(true).await;
    sequencer_loop_handle.abort();
    seq_http_server_handle.stop(true).await;

    Ok(())
}
