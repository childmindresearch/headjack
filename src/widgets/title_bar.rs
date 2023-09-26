pub struct TitleBarWidget<'a> {
    pub filename: &'a str,
    pub modes: &'a [&'a str],
    pub mode_index: usize,
}

impl<'a> TitleBarWidget<'a> {
    pub fn new(title: &'a str, modes: &'a [&'a str], mode_index: usize) -> TitleBarWidget<'a> {
        Self {
            filename: title,
            modes: modes,
            mode_index: mode_index,
        }
    }
}

fn path_extension_all(path: &std::path::Path) -> String {
    let mut p = path.clone();
    let mut ext = String::new();
    while let Some(e) = p.extension() {
        ext = format!("{}.{}", e.to_str().unwrap(), ext);
        p = std::path::Path::new(p.file_stem().unwrap());
    }
    ext.pop();
    return ext;
}

impl<'a> tui::widgets::Widget for TitleBarWidget<'a> {
    fn render(self, area: tui::prelude::Rect, buf: &mut tui::prelude::Buffer) {
        // draw modes from the right
        let mut x = area.x + area.width as u16;
        let mut modes_width = 0;
        for (idx, mode) in self.modes.iter().enumerate().rev() {
            x -= mode.len() as u16;
            let style = if self.mode_index == idx {
                tui::style::Style::default()
                    .bg(tui::style::Color::Cyan)
                    .add_modifier(tui::style::Modifier::BOLD)
            } else {
                tui::style::Style::default()
            };
            buf.set_stringn(x, area.y, mode, mode.len(), style);
            x -= 1;
            modes_width += mode.len() + 1;
        }

        let filename = std::path::Path::new(self.filename);
        // filename directory
        let filename_dir = filename.parent().unwrap().to_str().unwrap().to_string() + "/";
        let filename_ext = ".".to_string() + &path_extension_all(filename);
        let filename_stem =
            &self.filename[filename_dir.len()..filename.to_str().unwrap().len() - filename_ext.len()];

        // draw title from the left
        let style_filename_dir = tui::style::Style::default().fg(tui::style::Color::DarkGray);
        let style_filename_stem = tui::style::Style::default()
            .fg(tui::style::Color::Cyan)
            .add_modifier(tui::style::Modifier::BOLD);
        let style_filename_ext = tui::style::Style::default().fg(tui::style::Color::DarkGray);

        if filename_dir.len() + filename_stem.len() + filename_ext.len()
            < area.width as usize - modes_width
        {
            let pos_dir = area.x;
            let pos_stem = pos_dir + filename_dir.len() as u16;
            let pos_ext = pos_stem + filename_stem.len() as u16;

            buf.set_string(pos_dir, area.y, filename_dir, style_filename_dir);
            buf.set_string(pos_stem, area.y, filename_stem, style_filename_stem);
            buf.set_string(pos_ext, area.y, filename_ext, style_filename_ext);
        } else if filename_stem.len() + filename_ext.len() < area.width as usize - modes_width {
            let pos_stem = area.x;
            let pos_ext = pos_stem + filename_stem.len() as u16;

            buf.set_string(pos_stem, area.y, filename_stem, style_filename_stem);
            buf.set_string(pos_ext, area.y, filename_ext, style_filename_ext);
        } else if filename_stem.len() < area.width as usize - modes_width {
            let pos_stem = area.x;

            buf.set_string(pos_stem, area.y, filename_stem, style_filename_stem);
        }
    }
}
