use crate::utils;

pub struct ColorBarWidget {
    pub color_map: utils::colors::ColorMap,
    pub color_mode: utils::colors::ColorMode,
    pub min: f64,
    pub max: f64,
}

impl ColorBarWidget {
    pub fn new(
        color_map: utils::colors::ColorMap,
        color_mode: utils::colors::ColorMode,
        min: f64, 
        max: f64
    ) -> Self {
        Self {
            color_map,
            color_mode,
            min,
            max,
        }
    }
}

impl tui::widgets::Widget for ColorBarWidget {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let color_mapper = self.color_map.get();
        for i in 0..area.width {
            let color = color_mapper.color(i as f64 / area.width as f64, self.color_mode);
            buf.get_mut(area.x + i, area.y).set_bg(color);
        }

        // write min max values
        let min_str = format!("{:.2}", self.min);
        let max_str = format!("{:.2}", self.max);
        buf.set_stringn(
            area.x,
            area.y,
            &min_str,
            min_str.len(),
            tui::style::Style::default(),
        );
        buf.set_stringn(
            area.x + area.width as u16 - max_str.len() as u16,
            area.y,
            &max_str,
            max_str.len(),
            tui::style::Style::default(),
        );

        // write middle value
        let middle_str = format!("{:.2}", (self.min + self.max) / 2.);
        buf.set_stringn(
            area.x + area.width as u16 / 2 - middle_str.len() as u16 / 2,
            area.y,
            &middle_str,
            middle_str.len(),
            tui::style::Style::default(),
        );

    }
}
