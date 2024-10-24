mod anki_config;
mod anki_error_handling;
mod ankiconnect;
mod env_variables;
mod main_application;
mod menu_actions;
mod paths;
mod possible_entries;
mod screenshot;
mod tui_windows;
mod ui;

use crate::env_variables::{get_terminal_args, get_terminal_binary_name};
use crate::main_application::run_terminal_application;
use crate::screenshot::{capture_screenshot, create_unique_screenshot_filename, save_image};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    main: bool,

    #[arg(long)]
    tmp_dir: Option<PathBuf>,

    #[arg(long)]
    screenshot_path: Option<PathBuf>,

    #[arg(long)]
    page_number: Option<u32>,

    #[arg(long)]
    book_filename: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.main {
        let tmp_dir = args
            .tmp_dir
            .ok_or_else(|| anyhow!("Missing tmp_dir argument"))?;
        run_terminal_application(
            tmp_dir,
            args.screenshot_path,
            args.page_number,
            args.book_filename,
        )?;
    } else {
        let screenshot = capture_screenshot()?;

        let tmp_dir = create_tmp_dir()?;
        let screenshot_fn = create_unique_screenshot_filename();
        let screenshot_path = tmp_dir.as_ref().join(screenshot_fn);

        let mut main_application =
            spawn_terminal_with_main_process(tmp_dir.as_ref(), &screenshot_path, args)?;
        save_image(&screenshot, &screenshot_path)?;

        main_application.wait()?; // Must wait so that tmp_dir isn't cleaned up
    }

    Ok(())
}

fn create_tmp_dir() -> std::io::Result<TempDir> {
    tempfile::Builder::new().prefix("bookmining").tempdir()
}

fn spawn_terminal_with_main_process(
    tmp_dir: &Path,
    screenshot_path: &Path,
    args: Args,
) -> Result<Child> {
    let terminal_name = get_terminal_binary_name();
    let terminal_args = get_terminal_args();

    let mut command = Command::new(terminal_name);
    terminal_args.iter().for_each(|arg| {
        command.arg(arg);
    });

    // Application arguments
    command
        .arg("-e")
        .arg(std::env::current_exe()?)
        .arg("--main")
        .arg("--tmp-dir")
        .arg(tmp_dir)
        .arg("--screenshot-path")
        .arg(screenshot_path);

    if let Some(page_number) = args.page_number {
        command.arg("--page-number").arg(page_number.to_string());
    }

    if let Some(pdf_name) = args.book_filename {
        command.arg("--book-filename").arg(pdf_name);
    }

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    command
        .spawn()
        .context("Spawning terminal with main process")
}
