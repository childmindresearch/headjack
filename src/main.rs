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


static LONG_ABOUT: &'static str =
"headjack - Interactive NIfTI Viewer for the Terminal\n\n\
Interactive controls:\n\
\t- Arrow keys / WSAD: Move slice (Post.-Ant. / Left-Right)\n\
\t\tNavigate metadata\n\
\t- ZX: Move slice (Inf.-Sup.)\n\
\t- Tab: Toggle metadata view\n\
\t- C: Toggle color map\n\
\t- Q / Esc / Ctrl+C: Quit";

/// headjack - Interactive NIfTI Viewer for the Terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=LONG_ABOUT)]
struct Args {
    /// Image file name (.nii or .nii.gz)
    #[arg(index = 1)]
    input: String,

    /// ANSI color mode for terminals not supporting true color (24bit).
    #[arg(short, long, action)]
    ansi: bool,
    
    /// Verbose (debug) output.
    #[arg(short, long, action)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    // Read args
    let args = Args::parse();

    let color_mode = if args.ansi {
        ColorMode::Ansi256
    } else {
        ColorMode::TrueColor
    };

    // Create an application.
    let mut app = App::new(args.verbose, &args.input, color_mode)
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
