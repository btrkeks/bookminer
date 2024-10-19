use std::fs;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use crate::paths::get_anki_config_cache_file;
use crate::possible_entries::PossibleContent;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnkiConfig {
    pub deck_name: String,
    pub note_type: String,
    pub field_mapping: Vec<(String, PossibleContent)>
}

pub fn save_anki_config(config: &AnkiConfig) -> Result<()> {
    let cache_file = get_anki_config_cache_file()?;

    let config_json = serde_json::to_string_pretty(config)
        .with_context(|| anyhow!("Converting Anki config to a string"))?;

    fs::write(&cache_file, config_json)
        .with_context(|| anyhow!("Writing Anki config to file"))
}

pub fn load_anki_config() -> Result<Option<AnkiConfig>> {
    let cache_file = get_anki_config_cache_file()?;

    if !cache_file.exists() {
        return Ok(None);
    }

    let config_json = fs::read_to_string(&cache_file)
        .with_context(|| anyhow!("Reading Anki config from file"))?;

    let config = serde_json::from_str(&config_json)
        .with_context(|| anyhow!("Parsing stored Anki config"))?;

    Ok(Some(config))
}