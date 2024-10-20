use crate::anki_config::save_anki_config;
use crate::anki_error_handling::check_should_retry;
use crate::ankiconnect;
use crate::main_application::ApplicationState;
use crate::tui_windows::{
    edit_back, edit_front, select_anki_deck, select_anki_note_type,
    select_field_mapping_for_note_type,
};
use anyhow::{Context, Result};
use std::collections::HashMap;

pub trait MenuAction {
    fn new() -> Self
    where
        Self: Sized;

    // Perform the action
    fn act(&mut self, state: &mut ApplicationState) -> Result<()>;

    // Indicates whether the application should close after performing the action
    // or display the menu again
    fn should_exit(&self) -> bool;
}

pub struct CancelAction;
impl MenuAction for CancelAction {
    fn new() -> Self {
        CancelAction {}
    }

    fn act(&mut self, _state: &mut ApplicationState) -> Result<()> {
        Ok(())
    }
    fn should_exit(&self) -> bool {
        true
    }
}

pub struct SendCardAction {
    // should_quit: bool,
}

impl MenuAction for SendCardAction {
    fn new() -> Self {
        Self {
            // should_quit: false
        }
    }

    fn act(&mut self, state: &mut ApplicationState) -> Result<()> {
        let field_content = Self::get_field_contents_for_mapping(state)?;

        let mut files_to_send = Vec::with_capacity(1);
        if let Some(screenshot_path) = &state.screenshot_path {
            files_to_send.push(screenshot_path);
        }

        loop {
            match ankiconnect::send_note(
                &state.anki_config.deck_name,
                &state.anki_config.note_type,
                &field_content,
                &state.selected_tags,
                &files_to_send,
            ) {
                Ok(_) => return Ok(()),
                Err(e) => check_should_retry(e, &mut state.tui)?,
            }
        }
    }
    fn should_exit(&self) -> bool {
        true
    }
}

impl SendCardAction {
    fn get_field_contents_for_mapping(state: &ApplicationState) -> Result<HashMap<String, String>> {
        let field_mapping = &state.anki_config.field_mapping;
        let mut field_contents = HashMap::with_capacity(field_mapping.len());

        for (field_name, content_type) in field_mapping {
            let content = content_type.get_anki_card_content(state)?;
            field_contents.insert(field_name.clone(), content);
        }

        Ok(field_contents)
    }
}

pub struct EditAnkiSettings {}
impl MenuAction for EditAnkiSettings {
    fn new() -> Self {
        Self {}
    }

    fn act(&mut self, state: &mut ApplicationState) -> Result<()> {
        let settings = vec!["deck", "note type", "mapping", "cancel"];

        loop {
            let selection = state
                .tui
                .show_single_selection_menu("Choose the setting to change", &settings)?;

            match selection {
                0 => self.edit_deck(state)?,
                1 => self.edit_note_type(state)?,
                2 => self.edit_field_mapping(state)?,
                _ => break,
            }
        }

        save_anki_config(&state.anki_config).context("Saving updated Anki config")?;
        Ok(())
    }
    fn should_exit(&self) -> bool {
        false
    }
}

impl EditAnkiSettings {
    fn edit_deck(&self, state: &mut ApplicationState) -> Result<()> {
        let new_deck = select_anki_deck(&mut state.tui)?;
        state.anki_config.deck_name = new_deck;
        Ok(())
    }

    fn edit_note_type(&self, state: &mut ApplicationState) -> Result<()> {
        let new_note_type = select_anki_note_type(&mut state.tui)?;
        if new_note_type != state.anki_config.note_type {
            state.anki_config.note_type = new_note_type;
            self.edit_field_mapping(state)?;
        }
        Ok(())
    }

    fn edit_field_mapping(&self, state: &mut ApplicationState) -> Result<()> {
        let new_field_mapping =
            select_field_mapping_for_note_type(&mut state.tui, &state.anki_config.note_type)?;
        state.anki_config.field_mapping = new_field_mapping;
        Ok(())
    }
}

pub struct EditFrontAction {}
impl MenuAction for EditFrontAction {
    fn new() -> Self {
        Self {}
    }

    fn act(&mut self, state: &mut ApplicationState) -> Result<()> {
        edit_front(&mut state.tui, &state.tmp_dir)
    }
    fn should_exit(&self) -> bool {
        false
    }
}

pub struct EditBackAction {}
impl MenuAction for EditBackAction {
    fn new() -> Self {
        Self {}
    }

    fn act(&mut self, state: &mut ApplicationState) -> Result<()> {
        edit_back(&mut state.tui, &state.tmp_dir)
    }
    fn should_exit(&self) -> bool {
        false
    }
}
