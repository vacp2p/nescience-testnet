use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use sequencer_core::config::SequencerConfig;

pub fn from_file(config_home: PathBuf) -> Result<SequencerConfig> {
    let file = File::open(config_home)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}
