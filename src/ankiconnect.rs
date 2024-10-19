use std::{fs, io};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::exit;
use anyhow::{Result, Context};
use base64::Engine;
use base64::engine::general_purpose;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnkiConnectError {
    #[error("AnkiConnect is not running or unreachable")]
    NotRunning,

    #[error("Invalid filename: {0}")]
    InvalidFilename(String),

    #[error("AnkiConnect error: {0}")]
    AnkiConnectError(String),

    #[error("Failed to parse AnkiConnect response: {0}")]
    ParsingError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("File error: {action} '{path}': {source}")]
    FileError {
        action: String,
        path: PathBuf,
        source: io::Error,
    },
}

fn send_request(action: &str, params: Value) -> Result<Value, AnkiConnectError> {
    let client = Client::new();
    const URL: &str = "http://localhost:8765";
    let request_body = json!({
        "action": action,
        "version": 6,
        "params": params
    });

    match client.post(URL).json(&request_body).send() {
        Ok(response) => {
            let result: Value = response.json()
                .map_err(|e| AnkiConnectError::HttpError(e))?;

            match result.get("error") {
                Some(error) if !error.is_null() => {
                    Err(AnkiConnectError::AnkiConnectError(error.to_string()))
                },
                _ => Ok(result["result"].clone()),
            }
        },
        Err(e) => {
            if e.is_connect() {
                Err(AnkiConnectError::NotRunning)
            } else {
                Err(AnkiConnectError::HttpError(e))
            }
        }
    }
}

pub fn store_file(filepath: &Path) -> Result<(), AnkiConnectError> {
    let filename = filepath.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AnkiConnectError::InvalidFilename(filepath.display().to_string()))?;

    let file_content = fs::read(filepath).map_err(|e| AnkiConnectError::FileError {
        action: "reading".to_string(),
        path: filepath.to_path_buf(),
        source: e,
    })?;

    let params = json!({
        "filename": filename,
        "data":  general_purpose::STANDARD.encode(file_content),
        "deleteExisting": false,
    });

    send_request("storeMediaFile", params)?;

    // TODO: If there is a file with the same name (highly unlikely), Anki will rename the file.
    //       The response will contain the name of the renamed file
    Ok(())
}

fn create_add_note_params(deck: &str, note_type: &str, contents: &HashMap<String, String>, tags: &Vec<String>) -> Value {
    json!({
        "note": {
            "deckName": deck,
            "modelName": note_type,
            "fields": contents,
            "tags": tags,
            "options": {
                "allowDuplicate": true,
                "duplicateScope": "deck"
            }
        }
    })
}

pub fn send_note(deck: &str, note_type: &str, contents: &HashMap<String, String>, tags: &Vec<String>,
                 files: &Vec<&PathBuf>) -> Result<(), AnkiConnectError> {
    files.iter().try_for_each(|file| store_file(file) )?;
    let params = create_add_note_params(deck, note_type, contents, &tags);
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_add_note_params_basic() {
        let deck = "Test Deck";
        let note_type = "Basic";
        let mut contents = HashMap::new();
        contents.insert("Front".to_string(), "Test front".to_string());
        contents.insert("Back".to_string(), "Test back".to_string());
        let tags = vec!["test".to_string(), "example".to_string()];

        let result = create_add_note_params(deck, note_type, &contents, &tags);

        let expected = json!({
            "note": {
                "deckName": "Test Deck",
                "modelName": "Basic",
                "fields": {
                    "Front": "Test front",
                    "Back": "Test back"
                },
                "tags": ["test", "example"],
                "options": {
                    "allowDuplicate": true,
                    "duplicateScope": "deck"
                }
            }
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_add_note_params_empty_fields() {
        let deck = "Empty Deck";
        let note_type = "Empty";
        let contents = HashMap::new();
        let tags: Vec<String> = vec![];

        let result = create_add_note_params(deck, note_type, &contents, &tags);

        let expected = json!({
            "note": {
                "deckName": "Empty Deck",
                "modelName": "Empty",
                "fields": {},
                "tags": [],
                "options": {
                    "allowDuplicate": true,
                    "duplicateScope": "deck"
                }
            }
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_add_note_params_multiple_fields() {
        let deck = "Multi Field Deck";
        let note_type = "Complex";
        let mut contents = HashMap::new();
        contents.insert("Field1".to_string(), "Content1".to_string());
        contents.insert("Field2".to_string(), "Content2".to_string());
        contents.insert("Field3".to_string(), "Content3".to_string());
        let tags = vec!["complex".to_string()];

        let result = create_add_note_params(deck, note_type, &contents, &tags);

        let expected = json!({
            "note": {
                "deckName": "Multi Field Deck",
                "modelName": "Complex",
                "fields": {
                    "Field1": "Content1",
                    "Field2": "Content2",
                    "Field3": "Content3"
                },
                "tags": ["complex"],
                "options": {
                    "allowDuplicate": true,
                    "duplicateScope": "deck"
                }
            }
        });

        assert_eq!(result, expected);
    }
}