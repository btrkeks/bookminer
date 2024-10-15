mod main_application;
mod screenshot;
mod menu_actions;
mod paths;
mod edit_files;
mod ankiconnect;

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use std::process::{Child, Command};
use clap::Parser;
use crate::main_application::run_main_application;
use crate::screenshot::take_screenshot;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    main: bool,

    #[arg(long)]
    tmp_dir: Option<PathBuf>,

    #[arg(long)]
    screenshot_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.main {
        if let (Some(tmp_dir), Some(screenshot_path)) = (args.tmp_dir, args.screenshot_path) {
            run_main_application(tmp_dir, screenshot_path)?;
        } else {
            anyhow::bail!("Missing tmp_dir or screenshot_path arguments");
        }
    } else {
        let tmp_dir = tempfile::Builder::new().prefix("bookmining").tempdir()?;
        let screenshot_path = take_screenshot(tmp_dir.as_ref())?;
        let mut child = spawn_terminal_with_main_process(tmp_dir.as_ref(), &screenshot_path)?;

        child.wait()?; // Need to wait such that tmp_dir isn't cleaned
    }

    Ok(())
}

fn spawn_terminal_with_main_process(tmp_dir: &Path, screenshot_path: &Path) -> Result<Child> {
    Command::new("st")
        .arg("-n")
        .arg("floatterm")
        .arg("-g")
        .arg("90x25")
        .arg("-e")
        .arg(std::env::current_exe()?)
        .arg("--main")
        .arg("--tmp-dir")
        .arg(tmp_dir)
        .arg("--screenshot-path")
        .arg(screenshot_path)
        .spawn()
        .context("Spawning terminal with main process")
}