use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

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
        _ => {}
    }

    match app.mode {
        crate::app::AppMode::Xyz => {
            match key_event.code {
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
                KeyCode::Char('x') => {
                    app.decrement_slice(0);
                }
                KeyCode::Char('z') | KeyCode::Char('y') => {
                    app.increment_slice(0);
                }
                KeyCode::Tab => {
                    app.toggle_tab();
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    app.toggle_color_map();
                }
                _ => {}
            }
        }
        crate::app::AppMode::MetaData => {
            match key_event.code {
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Down | KeyCode::Char('s')  => {
                    app.increment_metadata_index();
                }
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Up | KeyCode::Char('w') => {
                    app.decrement_metadata_index();
                }
                KeyCode::Tab => {
                    app.toggle_tab();
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}

pub fn handle_mouse_events(_mouse_event: MouseEvent, _app: &mut App) -> AppResult<()> {
    /*match mouse_event.kind {
        crossterm::event::MouseEventKind::Down(button) => {
            match button {
                crossterm::event::MouseButton::Left => {
                    println!("row: {}, column: {}", mouse_event.row, mouse_event.column);
                },
                _ => {}
            }
        },
        _ => {}
        /*crossterm::event::MouseEventKind::Up(_) => {},
        crossterm::event::MouseEventKind::Drag(_) => todo!(),
        crossterm::event::MouseEventKind::Moved => todo!(),
        crossterm::event::MouseEventKind::ScrollDown => todo!(),
        crossterm::event::MouseEventKind::ScrollUp => todo!(),*/
    }*/
    Ok(())
}