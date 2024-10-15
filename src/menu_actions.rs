use std::path::Path;
use crate::edit_files::{edit_back, edit_front};
use anyhow::Result;

pub trait MenuAction {
    fn new() -> Self where Self: Sized;

    // Perform the action
    fn act(&self, tmp_dir: &Path, screenshot_path: &Path) -> Result<()>;

    // Indicates whether the application should close after performing the action
    // or display the menu again
    fn should_exit(&self) -> bool;
}

pub struct CancelAction;
impl MenuAction for CancelAction {
    fn new() -> Self {
        CancelAction {}
    }

    fn act(&self, _tmp_dir: &Path, _screenshot_path: &Path) -> Result<()> {
        Ok(())
    }
    fn should_exit(&self) -> bool {
        true
    }
}

pub struct SendCardAction {
    send_succeeded: bool,
}
impl MenuAction for SendCardAction {
    fn new() -> Self {
        Self {
            send_succeeded: false,
        }
    }

    fn act(&self, tmp_dir: &Path, screenshot_path: &Path) -> Result<()> {
        unimplemented!();
    }
    fn should_exit(&self) -> bool {
        self.send_succeeded
    }
}

pub struct EditFrontAction {
}
impl MenuAction for EditFrontAction {
    fn new() -> Self {
        Self {}
    }

    fn act(&self, tmp_dir: &Path, _screenshot_path: &Path) -> Result<()> {
        edit_front(tmp_dir)
    }
    fn should_exit(&self) -> bool {
        false
    }
}

pub struct EditBackAction {
}
impl MenuAction for EditBackAction {
    fn new() -> Self {
        Self {}
    }

    fn act(&self, tmp_dir: &Path, _screenshot_path: &Path) -> Result<()> {
        edit_back(tmp_dir)
    }
    fn should_exit(&self) -> bool {
        false
    }
}