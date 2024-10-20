use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn get_project_data_dir() -> Result<PathBuf> {
    let mut path = dirs::data_local_dir().ok_or_else(|| anyhow!("Getting home directory"))?;

    path.push("bookminer");

    fs::create_dir_all(&path).context("Creating project data directory")?;

    Ok(path)
}

pub fn get_tags_file() -> Result<PathBuf> {
    Ok(get_project_data_dir()?.join("tags"))
}

pub fn get_anki_config_cache_file() -> Result<PathBuf> {
    Ok(get_project_data_dir()?.join("last_selection"))
}
