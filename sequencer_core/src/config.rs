use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerConfig {
    ///Home dir of sequencer storage
    pub home: PathBuf,
    ///Override rust log (env var logging level)
    pub override_rust_log: Option<String>,
    ///Genesis id
    pub genesis_id: u64,
    ///If `True`, then adds random sequence of bytes to genesis block
    pub is_genesis_random: bool,
    ///Maximum number of transactions in block
    pub max_num_tx_in_block: usize,
    ///Interval in which blocks produced
    pub block_create_timeout_millis: Duration,
}
