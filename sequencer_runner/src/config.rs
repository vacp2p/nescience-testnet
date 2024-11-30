use std::path::PathBuf;

use anyhow::Result;
use sequencer_core::config::SequencerConfig;

use std::fs::File;
use std::io::BufReader;

pub fn from_file(config_home: PathBuf) -> Result<SequencerConfig> {
    let file = File::open(config_home)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}
