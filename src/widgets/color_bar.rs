use crate::utils::colors::colorous2tui;

pub struct ColorBarWidget<'a> {
    pub title: &'a str,
    pub gradient: colorous::Gradient,
    pub min: f32,
    pub max: f32,
}

impl<'a> ColorBarWidget<'a> {
    pub fn new(title: &'a str, gradient: colorous::Gradient, min: f32, max: f32) -> Self {
        Self {
            title,
            gradient,
            min,
            max,
        }
    }
}

impl tui::widgets::Widget for ColorBarWidget<'_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        for i in 0..area.width {
            let color = self.gradient.eval_rational(i as usize, area.width as usize);
            buf.get_mut(area.x + i, area.y).set_bg(colorous2tui(color));
        }

        // write title in the middle
        let title_len = self.title.len();
        let title_start = (area.width - title_len as u16) / 2;
        buf.set_stringn(
            area.x + title_start as u16,
            area.y,
            self.title,
            title_len,
            tui::style::Style::default(),
        );

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
    }
}
