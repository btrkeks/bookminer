use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use base64::Engine;
use base64::engine::general_purpose;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::paths::get_anki_config_cache_file;

#[derive(Error, Debug)]
pub enum AnkiConnectError {
    #[error("AnkiConnect is not running")]
    NotRunning,
    #[error("Invalid filename")]
    InvalidFilename,
    #[error("AnkiConnect error: {0}")]
    AnkiConnectError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

fn send_request(action: &str, params: Value) -> Result<Value> {
    let client = Client::new();
    let response = client.post("http://localhost:8765")
        .json(&json!({
            "action": action,
            "version": 6,
            "params": params
        }))
        .send()
        .context("Sending request to AnkiConnect")?;

    let result: Value = response.json().context("Parsing AnkiConnect response")?;

    if let Some(error) = result.get("error") {
        anyhow::bail!("AnkiConnect error: {}", error);
    }

    Ok(result["result"].clone())
}

fn store_file(filepath: &Path) -> Result<String, AnkiConnectError> {
    let filename = filepath.file_name()
        .and_then(|name| name.to_str())
        .ok_or(AnkiConnectError::InvalidFilename)?;

    let file_content = fs::read(filepath)?;

    let params = json!({
        "filename": filename,
        "data":  general_purpose::STANDARD.encode(file_content)
    });

    let result = send_request("storeMediaFile", params)?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

struct FieldMapping {
    field: String,
    value: String,
}

pub fn send_note(anki_config: AnkiConfig, contents: Vec<FieldMapping>, tags: Vec<String>) -> Result<(), AnkiConnectError> {
    let fields: Value = contents.into_iter()
        .map(|fm| (fm.field, fm.value))
        .collect();

    let params = json!({
        "note": {
            "deckName": anki_config.deck_name,
            "modelName": anki_config.note_type,
            "fields": fields,
            "tags": tags,
            "options": {
                "allowDuplicate": false,
                "duplicateScope": "deck"
            }
        }
    });

    send_request("addNote", params)?;
    Ok(())
}

pub fn get_deck_names() -> Result<Vec<String>, AnkiConnectError> {
    let result = send_request("deckNames", json!({}))?;
    result.as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .ok_or_else(|| AnkiConnectError::AnkiConnectError("Invalid response format".to_string()))
}

pub fn get_model_names() -> Result<Vec<String>, AnkiConnectError> {
    let result = send_request("modelNames", json!({}))?;
    result.as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .ok_or_else(|| AnkiConnectError::AnkiConnectError("Invalid response format".to_string()))
}

pub fn get_field_names(model_name: &str) -> Result<Vec<String>, AnkiConnectError> {
    let params = json!({
        "modelName": model_name
    });
    let result = send_request("modelFieldNames", params)?;
    result.as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .ok_or_else(|| AnkiConnectError::AnkiConnectError("Invalid response format".to_string()))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnkiConfig {
    pub deck_name: String,
    pub note_type: String,
}

pub fn save_anki_config(config: &AnkiConfig) -> Result<(), AnkiConnectError> {
    let cache_file = get_anki_config_cache_file()?;

    let config_json = serde_json::to_string_pretty(config)
        .map_err(|e| AnkiConnectError::Other(e.into()))?;

    fs::write(&cache_file, config_json)
        .map_err(AnkiConnectError::IoError)
}

pub fn load_anki_config() -> Result<Option<AnkiConfig>, AnkiConnectError> {
    let cache_file = get_anki_config_cache_file()?;

    if !cache_file.exists() {
        return Ok(None);
    }

    let config_json = fs::read_to_string(&cache_file)
        .map_err(AnkiConnectError::IoError)?;

    let config = serde_json::from_str(&config_json)
        .map_err(|e| AnkiConnectError::Other(e.into()))?;

    Ok(Some(config))
}