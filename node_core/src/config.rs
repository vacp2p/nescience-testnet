use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    ///Home dir of sequencer storage
    pub home: PathBuf,
    ///Override rust log (env var logging level)
    pub override_rust_log: Option<String>,
    ///Sequencer URL
    pub sequencer_addr: String,
    ///Sequencer polling duration for new blocks in seconds
    pub seq_poll_timeout_secs: u64,
}
