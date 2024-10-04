use std::sync::Arc;

use anyhow::Result;
use consensus::ConsensusManager;
use log::info;
use networking::peer_manager::PeerManager;
use node_rpc::new_http_server;
use rpc_primitives::RpcConfig;
use tokio::sync::Mutex;

pub async fn main_runner() -> Result<()> {
    env_logger::init();

    let http_server = new_http_server(RpcConfig::default())?;
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
