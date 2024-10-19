use std::path::{Path, PathBuf};
use anyhow::Context;
use chrono::Local;
use screenshots::image::RgbaImage;
use screenshots::Screen;
use anyhow::Result;

pub fn create_unique_screenshot_filename() -> String {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    format!("screenshot_{}.png", timestamp)
}

pub(crate) fn capture_screenshot() -> Result<RgbaImage> {
    let screens = Screen::all()?;
    let screen = screens.first().context("No screen found")?;

    let image = screen.capture()
        .context("Capturing screen")?;
    Ok(image)
}

pub fn save_image(image: &RgbaImage, path: &Path) -> Result<()> {
    image.save(&path)
        .context("Saving screenshot")
}