use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};

use crate::{config::NodeConfig, HOME_DIR_ENV_VAR};

pub fn vec_u8_to_vec_u64(bytes: Vec<u8>) -> Vec<u64> {
    // Pad with zeros to make sure it's a multiple of 8
    let mut padded = bytes.clone();
    while !padded.len().is_multiple_of(8) {
        padded.push(0);
    }

    padded
        .chunks(8)
        .map(|chunk| {
            let mut array = [0u8; 8];
            array.copy_from_slice(chunk);
            u64::from_le_bytes(array)
        })
        .collect()
}

///Get home dir for wallet. Env var `NSSA_WALLET_HOME_DIR` must be set before execution to succeed.
pub fn get_home() -> Result<PathBuf> {
    Ok(PathBuf::from_str(&std::env::var(HOME_DIR_ENV_VAR)?)?)
}

///Fetch config from `NSSA_WALLET_HOME_DIR`
pub fn fetch_config() -> Result<NodeConfig> {
    let config_home = get_home()?;
    let file = File::open(config_home.join("node_config.json"))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}

//ToDo: Replace with structures conversion in future
pub fn produce_account_addr_from_hex(hex_str: String) -> Result<[u8; 32]> {
    hex::decode(hex_str)?
        .try_into()
        .map_err(|_| anyhow!("Failed conversion to 32 bytes"))
}
