use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        // Counter handlers
        KeyCode::Right | KeyCode::Char('d') => {
            app.increment_slice(2);
        }
        KeyCode::Left | KeyCode::Char('a') => {
            app.decrement_slice(2);
        }
        KeyCode::Up | KeyCode::Char('w') => {
            app.increment_slice(1);
        }
        KeyCode::Down | KeyCode::Char('s') => {
            app.decrement_slice(1);
        }
        KeyCode::Char('x') | KeyCode::Char('e') => {
            app.increment_slice(0);
        }
        KeyCode::Char('z') | KeyCode::Char('y') | KeyCode::Char('q') => {
            app.decrement_slice(0);
        }
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
