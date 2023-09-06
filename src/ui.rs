use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::{app::App, brain};

static COORDS: [&str; 3] = ["Superior", "Anterior", "Right"];

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    match app.mode {
        crate::app::AppMode::Xyz => {
            let main_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                    ]
                    .as_ref(),
                )
                .split(frame.size());

            for i in 0..3 {
                let display_axis = i;
                frame.render_widget(
                    brain::SliceWidget::new(
                        &mut app.image_sampler,
                        &mut app.image_cache,
                        app.intensity_range,
                        app.slice_position.clone(),
                        display_axis,
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!(
                                "{} = {}",
                                COORDS[display_axis], app.slice_position[display_axis]
                            ))
                            .border_type(BorderType::Rounded),
                    ),
                    main_layout[i],
                );
            }
        }
        crate::app::AppMode::MetaData => {
            frame.render_widget(brain::MetaDataWidget::new(&app.image_sampler), frame.size());
        }
    }
}
