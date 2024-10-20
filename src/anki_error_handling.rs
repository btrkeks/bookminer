use crate::ankiconnect::AnkiConnectError;
use crate::ui::tui::Tui;

pub fn check_should_retry(
    error: AnkiConnectError,
    tui: &mut Tui,
) -> anyhow::Result<(), AnkiConnectError> {
    match error {
        AnkiConnectError::NotRunning => {
            match tui.show_dialog("Anki is not running. Do you want to retry?") {
                Ok(true) => Ok(()),                 // Retry
                Ok(false) => std::process::exit(1), // Quit
                Err(_) => Err(error),               // Dialog error, propagate the original error
            }
        }
        _ => Err(error),
    }
}
