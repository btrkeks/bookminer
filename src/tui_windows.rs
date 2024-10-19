use std::{fs, io};
use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};
use crate::anki_error_handling::handle_anki_connect_error;
use crate::ankiconnect::{get_deck_names, get_field_names, get_model_names};
use crate::env_variables::{get_editor_binary_name};
use crate::main_application::ApplicationState;
use crate::menu_actions::{CancelAction, EditAnkiSettings, EditBackAction, EditFrontAction, MenuAction, SendCardAction};
use crate::possible_entries::PossibleContent;
use crate::ui::tui::Tui;

fn edit_file(file_name: &str, tmp_dir: &Path) -> io::Result<()> {
    let file_path = tmp_dir.join(file_name);

    if !file_path.exists() {
        fs::File::create(&file_path)?;
    }

    let editor_name = get_editor_binary_name();
    Command::new(editor_name)
        .arg(file_path)
        .status()?;
    Ok(())
}

pub fn edit_front(tui: &mut Tui, tmp_dir: &Path) -> Result<()> {
    let path= tmp_dir.join("front.tex");
    tui.edit_file(&path)
        .context("Editing front file")
}

pub fn edit_back(tui: &mut Tui, tmp_dir: &Path) -> Result<()> {
    let path= tmp_dir.join("back.tex");
    tui.edit_file(&path)
        .context("Editing front file")
}

pub fn select_anki_deck(tui: &mut Tui) -> Result<String> {
    loop {
        match get_deck_names() {
            Ok(decks) =>{
                let index = tui.show_single_selection_menu("Select Anki Deck", &decks)?;
                return Ok(decks[index].clone()); // TODO: Is cloning necessary?
            },
            Err(e) => handle_anki_connect_error(e, tui)?,
        }
    }
}

pub fn select_anki_note_type(tui: &mut Tui) -> Result<String> {
    loop {
        match get_model_names() {
            Ok(note_types) =>{
                let index = tui.show_single_selection_menu("Select Anki Note Type", &note_types)?;
                return Ok(note_types[index].clone()); // TODO: Is cloning necessary?
            },
            Err(e) => handle_anki_connect_error(e, tui)?,
        }
    }
}

pub fn select_from_possible_content(tui: &mut Tui, field_name: &str) -> Result<PossibleContent> {
    let options = vec![
        "Empty",
        "Front",
        "Back",
        "Screenshot",
        "Page Number",
        "File Name",
    ];

    let title = format!("Choose the contents for the field {}", field_name);
    let selected = tui.show_single_selection_menu(&title, &options)?;

    Ok(match selected {
        0 => PossibleContent::Empty,
        1 => PossibleContent::Front,
        2 => PossibleContent::Back,
        3 => PossibleContent::Screenshot,
        4 => PossibleContent::PageNumber,
        5 => PossibleContent::FileName,
        _ => unreachable!(),
    })
}

pub fn select_field_mapping_for_note_type(tui: &mut Tui, note_type: &str) -> Result<Vec<(String, PossibleContent)>> {
    let field_names;
    loop {
        match get_field_names(&note_type) {
            Ok(result) =>{
                field_names = result;
                break;
            },
            Err(e) => handle_anki_connect_error(e, tui)?,
        }
    }

    let mut field_mapping: Vec<(String, PossibleContent)> = Vec::with_capacity(field_names.len());
    for field_name in field_names {
        let selection: PossibleContent = select_from_possible_content(tui, &field_name)?;
        field_mapping.push((field_name, selection));
    }

    Ok(field_mapping)
}

pub fn show_final_menu(state: &mut ApplicationState) -> Result<Box<dyn MenuAction>> {
    let menu_items = vec!["Send Card", "Edit Front", "Edit Back", "Edit Anki Settings", "Edit Tags", "Cancel"];

    let selected = state.tui.show_single_selection_menu("Menu", &menu_items)?;

    Ok(match selected {
        0 => Box::new(SendCardAction::new()),
        1 => Box::new(EditFrontAction::new()),
        2 => Box::new(EditBackAction::new()),
        3 => Box::new(EditAnkiSettings::new()),
        4 => unimplemented!(),
        _ => Box::new(CancelAction::new()),
    })
}