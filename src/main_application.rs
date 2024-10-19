use std::{fs};
use std::path::{Path, PathBuf};
use crate::menu_actions::{EditAnkiSettings, EditBackAction, EditFrontAction, MenuAction, SendCardAction};
use crate::paths::get_tags_file;
use anyhow::{Context, Result};
use crate::anki_config;
use crate::anki_config::{load_anki_config, save_anki_config, AnkiConfig};
use crate::tui_windows::{edit_back, edit_front, select_anki_deck, select_anki_note_type, select_field_mapping_for_note_type, show_final_menu};
use crate::menu_actions::CancelAction;
use crate::ui::tui::Tui;

fn save_tags(tags: &Vec<String>) -> Result<()> {
    let tags_file = get_tags_file()?;
    fs::write(&tags_file, tags.join("\n")).context("Storing tags")
}

fn load_tags() -> Result<Vec<String>> {
    let tags_file = get_tags_file()?;

    let tags = if tags_file.exists() {
        fs::read_to_string(&tags_file)?
            .lines()
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    Ok(tags)
}

fn display_anki_config(anki_config: &AnkiConfig) -> Result<()> {
    // Display the Anki config in a window next to the selection menu
    Ok(())
}

fn show_anki_not_running_dialog() {
    // Show a dialog with
    // "Could not connect to Anki. Check if Anki is running and AnkiConnect installed"
    // [Retry] [Quit]
}

fn ask_for_anki_config(tui: &mut Tui) -> Result<AnkiConfig> {
    let deck_name = select_anki_deck(tui)?;
    let note_type = select_anki_note_type(tui)?;
    let field_mapping = select_field_mapping_for_note_type(tui, &note_type)?;

    let anki_config = AnkiConfig {
        deck_name,
        note_type,
        field_mapping,
    };

    save_anki_config(&anki_config)?;
    Ok(anki_config)
}

pub struct ApplicationState {
    pub(crate) tui: Tui,
    pub(crate) selected_tags: Vec<String>,
    pub(crate) anki_config: AnkiConfig,
    pub(crate) screenshot_path: Option<PathBuf>,
    pub(crate) tmp_dir: PathBuf,
    pub(crate) page_number: Option<u32>,
    pub(crate) book_filename: Option<String>,
}

pub fn run_terminal_application(tmp_dir: PathBuf,
                                screenshot_path: Option<PathBuf>,
                                page_number: Option<u32>,
                                book_filename: Option<String>
) -> Result<()> {
    let mut tui = Tui::new()?;

    edit_front(&mut tui, &tmp_dir)?;
    edit_back(&mut tui, &tmp_dir)?;

    let mut tags = load_tags()?;
    let selected_tags = tui.show_tag_menu(&mut tags)?;
    save_tags(&tags)?;

    let anki_config = if let Some(ac) = load_anki_config()? {
        ac
    } else {
        ask_for_anki_config(&mut tui)?
    };

    let mut state = ApplicationState {
        tui,
        selected_tags,
        anki_config,
        screenshot_path,
        tmp_dir,
        page_number,
        book_filename,
    };

    while let mut chosen_action = show_final_menu(&mut state)? {
        chosen_action.act(&mut state)?;
        if chosen_action.should_exit() {
            break;
        }
    }

    Ok(())
}