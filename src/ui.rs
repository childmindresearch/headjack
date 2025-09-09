use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::{
    app::App,
    widgets::{
        color_bar::ColorBarWidget,
        key_value_list_widget::KeyValueListWidget,
        slice_widget::{SliceParams, XyzWidget},
        title_bar::TitleBarWidget,
    },
};

static MODE_TITLES: [&str; 2] = ["Voxel", "Metadata"];

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_>) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(frame.area());

    // write out filename

    let mode_index = match app.mode {
        crate::app::AppMode::Xyz => 0,
        crate::app::AppMode::MetaData => 1,
    };

    frame.render_widget(
        TitleBarWidget::new(&app.file_path, &MODE_TITLES, mode_index),
        layout[0],
    );

    frame.render_widget(
        ColorBarWidget::new(
            app.color_map,
            app.color_mode,
            app.intensity_range.0,
            app.intensity_range.1,
        ),
        layout[2],
    );

    match app.mode {
        crate::app::AppMode::Xyz => {
            let slice = SliceParams {
                position: app.slice_position.clone(),
                intensity_range: app.intensity_range,
                color_map: app.color_map,
                color_mode: app.color_mode,
            };
            frame.render_widget(
                XyzWidget::new(&app.volume, &mut app.image_cache, &slice),
                layout[1],
            );
        }
        crate::app::AppMode::MetaData => {
            frame.render_widget(
                KeyValueListWidget::new(&app.metadata, app.metadata_index),
                layout[1],
            );
        }
    }
}
