use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use consensus::ConsensusManager;
use log::info;
use networking::peer_manager::PeerManager;
use node_core::{config::NodeConfig, NodeCore};
use node_rpc::new_http_server;
use rpc_primitives::RpcConfig;
use tokio::sync::Mutex;

pub async fn main_runner() -> Result<()> {
    env_logger::init();

    //ToDo: Change it
    let node_config = NodeConfig {
        home: PathBuf::new(),
        override_rust_log: None,
        sequencer_addr: "addr".to_string(),
        seq_poll_timeout_secs: 1,
    };

    let node_core = NodeCore::start_from_config_update_chain(node_config.clone()).await?;
    let wrapped_node_core = Arc::new(Mutex::new(node_core));

    let http_server = new_http_server(
        RpcConfig::default(),
        node_config.clone(),
        wrapped_node_core.clone(),
    )?;
    info!("HTTP server started");
    let _http_server_handle = http_server.handle();
    tokio::spawn(http_server);

    let peer_manager = PeerManager::start_peer_manager(4, 0).await?;
    info!("Peer manager mock started");

    let peer_manager_shared = Arc::new(Mutex::new(peer_manager));

    let _consensus_manager = ConsensusManager::new(peer_manager_shared.clone());
    info!("Consensus manger mock started");

    #[allow(clippy::empty_loop)]
    loop {
        //ToDo: Insert activity into main loop
    }
}
