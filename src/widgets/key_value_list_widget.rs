use tui::style::{Style, Color, Modifier};

pub type KeyValueList = Vec<(String, String)>;

pub struct KeyValueListWidget<'a> {
    pub items: &'a KeyValueList,
    pub max_key_len: usize,
    pub max_value_len: usize,
    pub offset: usize,
    pub style_key: Style,
    pub style_value: Style,
}

impl<'a> KeyValueListWidget<'a> {
    pub fn new(items: &'a KeyValueList, offset: usize) -> Self {
        let max_key_len = items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        let max_value_len = items.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
        Self {
            items,
            max_key_len,
            max_value_len,
            offset,
            style_key: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            style_value: Style::default(),
        }
    }
}

impl<'a> tui::widgets::Widget for KeyValueListWidget<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        for (i, (key, val)) in self.items.iter().enumerate() {
            let i_offset: isize = i as isize - self.offset as isize;
            if (i_offset < 0) || (i_offset >= (area.height) as isize) {
                continue;
            }
            let i_offset: usize = i_offset as usize;

            if (i_offset == 0) && (self.offset > 0) {
                buf.set_stringn(
                    area.x + 1,
                    area.y + i_offset as u16,
                    "[...]",
                    area.width.into(),
                    self.style_value,
                );
            } else if i_offset < (area.height - 1) as usize {
                buf.set_stringn(
                    area.x,
                    area.y + i_offset as u16,
                    key,
                    self.max_key_len,
                    self.style_key,
                );
                buf.set_stringn(
                    area.x + self.max_key_len as u16 + 1,
                    area.y + i_offset as u16,
                    val,
                    self.max_value_len,
                    self.style_value,
                );
            } else if i_offset == (area.height - 1) as usize {
                buf.set_stringn(
                    area.x + 1,
                    area.y + i_offset as u16,
                    "[...]",
                    area.width.into(),
                    self.style_value,
                );
            }
        }
    }
}
