use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use screenshots::image::RgbaImage;
use screenshots::Screen;
use std::path::Path;

pub fn create_unique_screenshot_filename() -> String {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    format!("screenshot_{}.png", timestamp)
}

pub(crate) fn capture_screenshot() -> Result<RgbaImage> {
    let screens = Screen::all()?;
    let screen = screens.first().context("No screen found")?;

    let image = screen.capture().context("Capturing screen")?;
    Ok(image)
}

pub fn save_image(image: &RgbaImage, path: &Path) -> Result<()> {
    image.save(&path).context("Saving screenshot")
}
