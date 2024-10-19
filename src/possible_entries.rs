use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::ankiconnect;
use crate::main_application::ApplicationState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PossibleContent {
    Empty,
    Front,
    Back,
    Screenshot,
    PageNumber,
    FileName,
}

impl PossibleContent {
    pub fn get_anki_card_content(&self, state: &ApplicationState) -> Result<String> {
        match self {
            PossibleContent::Empty => {
                Ok("".to_string())
            }
            PossibleContent::Front => {
                let front_text = get_front_text(&state.tmp_dir)?;
                let latex_wrapped_text = format!("[latex]{}[/latex]", front_text);
                Ok(anki_escape_string(&latex_wrapped_text))
            }
            PossibleContent::Back => {
                let back_text = get_back_text(&state.tmp_dir)?;
                let latex_wrapped_text = format!("[latex]{}[/latex]", back_text);
                Ok(anki_escape_string(&latex_wrapped_text))
            }
            PossibleContent::Screenshot => {
                if let Some(screenshot_path) = &state.screenshot_path {
                    // ankiconnect::store_file(screenshot_path)?;

                    Ok(format!("<img src=\"{}\">", screenshot_path.file_name()
                        .ok_or_else(|| anyhow::anyhow!("Invalid screenshot filename"))?
                        .to_str()
                        .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 screenshot filename"))?
                    ))
                } else {
                    Ok("".to_string())
                }
            }
            PossibleContent::PageNumber => {
                if let Some(page_number) = state.page_number {
                    Ok(page_number.to_string())
                }
                else {
                    Ok(String::new())

                }
            }
            PossibleContent::FileName => {
                if let Some(filename) = &state.book_filename {
                    Ok(filename.clone())
                }
                else {
                    Ok(String::new())

                }
            }
        }
    }
}

fn get_front_text(tmp_dir: &Path) -> Result<String> {
    fs::read_to_string(tmp_dir.join("front.tex"))
        .context("Reading front.tex")
}

fn get_back_text(tmp_dir: &Path) -> Result<String> {
    fs::read_to_string(tmp_dir.join("back.tex"))
        .context("Reading back.tex")
}

fn anki_escape_string(string: &str) -> String {
    string
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\t', "&Tab;")
}