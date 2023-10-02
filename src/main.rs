use anyhow::{anyhow, Context};
use clap::Parser;
use headjack::app::App;
use headjack::event::{Event, EventHandler};
use headjack::handler::{handle_key_events, handle_mouse_events};
use headjack::tui::Tui;
use headjack::utils::colors::ColorMode;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

/// headjack - Terminal UI 3D volume brain imaging viewer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Image file name (.nii or .nii.gz)
    #[arg(index = 1)]
    input: String,

    /// ANSI color mode for terminals not supporting true color (24bit).
    #[arg(short, long, action)]
    ansi: bool,

    /// Black and white color mode for terminals not supporting any color (what year is this?).
    #[arg(short, long, action)]
    bw: bool,
}

fn main() -> anyhow::Result<()> {
    // Read args
    let args = Args::parse();

    let color_mode = if args.bw {
        ColorMode::Bw
    } else if args.ansi {
        ColorMode::Ansi256
    } else {
        ColorMode::TrueColor
    };

    // Create an application.
    let mut app = App::new(&args.input, color_mode)
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("Failed to load data '{}'", &args.input))?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("Failed to init terminal"))?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to draw"))?;
        // Handle events.
        match tui
            .events
            .next()
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to process event"))?
        {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)
                .map_err(|e| anyhow!(e))
                .with_context(|| format!("Failed to process key event"))?,
            Event::Mouse(mouse_event) => handle_mouse_events(mouse_event, &mut app)
                .map_err(|e| anyhow!(e))
                .with_context(|| format!("Failed to process mouse event"))?,
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("Failed to exit the interface"))?;
    Ok(())
}
