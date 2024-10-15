use std::path::{Path, PathBuf};
use anyhow::Context;
use chrono::Local;
use screenshots::image::RgbaImage;
use screenshots::Screen;

fn create_unique_screenshot_filename() -> String {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    format!("screenshot_{}.png", timestamp)
}

fn capture_screenshot() -> anyhow::Result<RgbaImage> {
    let screens = Screen::all()?;
    let screen = screens.first().context("No screen found")?;
    let image = screen.capture()?;
    Ok(image)
}

pub fn take_screenshot(dir: &Path) -> anyhow::Result<PathBuf> {
    let screenshot_fn = create_unique_screenshot_filename();
    let screenshot_path = dir.join(screenshot_fn);

    let image = capture_screenshot()?;
    image.save(&screenshot_path)?;

    Ok(screenshot_path)
}