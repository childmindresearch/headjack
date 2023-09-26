#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    TrueColor,
    Ansi256,
    Bw,
}

struct HjColor(colorous::Color);

impl ansi_colours::AsRGB for HjColor {
    fn as_u32(&self) -> u32 {
        ((self.0.r as u32) << 16) + ((self.0.g as u32) << 8) + self.0.b as u32
    }
}

impl Into<tui::style::Color> for HjColor {
    fn into(self) -> tui::style::Color {
        tui::style::Color::Rgb(self.0.r, self.0.g, self.0.b)
    }
}

impl HjColor {
    pub fn invert(&self) -> Self {
        Self {
            0: colorous::Color { 
                r: 255 - self.0.r,
                g: 255 - self.0.g,
                b: 255 - self.0.b,
            }
        }
    }
}

pub fn calc_termcolor(mode: ColorMode, map: colorous::Gradient, val: usize, val_max: usize) -> tui::style::Color {
    let rgb = HjColor(map.eval_rational(val, val_max));

    match mode {
        ColorMode::TrueColor => {
            rgb.into()
        },
        ColorMode::Ansi256 => {
            tui::style::Color::Indexed(ansi_colours::ansi256_from_rgb(rgb))
        },
        ColorMode::Bw => {
            if val > (val_max/2) { tui::style::Color::White } else { tui::style::Color::Black }
        },
    }
}

pub fn calc_termcolor_inverted(mode: ColorMode, map: colorous::Gradient, val: usize, val_max: usize) -> tui::style::Color {
    let rgb = HjColor(map.eval_rational(val, val_max)).invert();

    match mode {
        ColorMode::TrueColor => {
            rgb.into()
        },
        ColorMode::Ansi256 => {
            tui::style::Color::Indexed(ansi_colours::ansi256_from_rgb(rgb))
        },
        ColorMode::Bw => {
            if val <= (val_max/2) { tui::style::Color::White } else { tui::style::Color::Black }
        },
    }
}

pub fn colorous2tui(value: colorous::Color) -> tui::style::Color {
    tui::style::Color::Rgb(value.r, value.g, value.b)
}

pub fn invert_color(value: colorous::Color) -> colorous::Color {
    colorous::Color {
        r: 255 - value.r,
        g: 255 - value.g,
        b: 255 - value.b,
    }
}