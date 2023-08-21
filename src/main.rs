use headjack::app::{App, AppResult};
use headjack::event::{Event, EventHandler};
use headjack::handler::handle_key_events;
use headjack::tui::Tui;
use std::{io, env};
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> AppResult<()> {
    println!("Start.");

    // Read CLI arguments.
    let args: Vec<String> = env::args().collect();
    let arg_input = args.get(1).ok_or("No input file")?;
    
    println!("Create app.");

    // Create an application.
    let mut app = App::new(arg_input);

    
    println!("Init TUI.");

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
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
