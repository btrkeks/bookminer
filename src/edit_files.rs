use std::{fs, io};
use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};

fn edit_file(file_name: &str, tmp_dir: &Path) -> io::Result<()> {
    let file_path = tmp_dir.join(file_name);
    if !file_path.exists() {
        fs::File::create(&file_path)?;
    }
    Command::new("vim")
        .arg(file_path)
        .status()?;
    Ok(())
}

pub fn edit_front(tmp_dir: &Path) -> Result<()> {
    edit_file("front.tex", tmp_dir)
        .context("Editing front file")
}

pub fn edit_back(tmp_dir: &Path) -> Result<()> {
    edit_file("back.tex", tmp_dir)
        .context("Editing back file")
}