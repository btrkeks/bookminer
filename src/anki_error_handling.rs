use crate::ankiconnect::AnkiConnectError;
use crate::ui::tui::Tui;

pub fn handle_anki_connect_error(error: AnkiConnectError, tui: &mut Tui) -> anyhow::Result<(), AnkiConnectError> {
    match error {
        AnkiConnectError::NotRunning => {
            loop {
                match tui.show_dialog("Anki is not running. Do you want to retry?") {
                    Ok(true) => return Ok(()), // Retry
                    Ok(false) => std::process::exit(1), // Quit
                    Err(_) => return Err(error), // Dialog error, propagate the original error
                }
            }
        }
        _ => Err(error),
    }
}