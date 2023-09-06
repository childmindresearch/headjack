use clap::Parser;
use headjack::app::{App, AppResult};
use headjack::event::{Event, EventHandler};
use headjack::handler::{handle_key_events, handle_mouse_events};
use headjack::tui::Tui;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(index = 1)]
    input: String,
}

fn main() -> AppResult<()> {
    // Read args
    let args = Args::parse();

    // Create an application.
    let mut app = App::new(&args.input);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(mouse_event) => handle_mouse_events(mouse_event, &mut app)?,
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
