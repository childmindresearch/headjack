#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    TrueColor,
    Ansi256,
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
            },
        }
    }

    pub fn luma(&self) -> f64 {
        0.2126 * self.0.r as f64 + 0.7152 * self.0.g as f64 + 0.0722 * self.0.b as f64
    }
}

pub fn calc_termcolor_rational(
    mode: ColorMode,
    map: colorous::Gradient,
    val: usize,
    val_max: usize,
) -> tui::style::Color {
    let rgb = HjColor(map.eval_rational(val, val_max));

    match mode {
        ColorMode::TrueColor => rgb.into(),
        ColorMode::Ansi256 => tui::style::Color::Indexed(ansi_colours::ansi256_from_rgb(rgb)),
    }
}

pub fn calc_termcolor_continuous(
    mode: ColorMode,
    map: colorous::Gradient,
    val: f64,
) -> tui::style::Color {
    let rgb = HjColor(map.eval_continuous(val));

    match mode {
        ColorMode::TrueColor => rgb.into(),
        ColorMode::Ansi256 => tui::style::Color::Indexed(ansi_colours::ansi256_from_rgb(rgb)),
    }
}

pub fn calc_termcolor_inverted_rational(
    mode: ColorMode,
    map: colorous::Gradient,
    val: usize,
    val_max: usize,
) -> tui::style::Color {
    let rgb = HjColor(map.eval_rational(val, val_max)).invert();

    match mode {
        ColorMode::TrueColor => rgb.into(),
        ColorMode::Ansi256 => tui::style::Color::Indexed(ansi_colours::ansi256_from_rgb(rgb)),
    }
}

pub fn calc_termcolor_inverted_continuous(
    mode: ColorMode,
    map: colorous::Gradient,
    val: f64,
) -> tui::style::Color {
    let rgb = HjColor(map.eval_continuous(val)).invert();

    match mode {
        ColorMode::TrueColor => rgb.into(),
        ColorMode::Ansi256 => tui::style::Color::Indexed(ansi_colours::ansi256_from_rgb(rgb)),
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

#[derive(Debug, Clone, Copy)]
pub struct ColorMapper {
    pub gradient: colorous::Gradient,
    pub invert: bool,
}

impl ColorMapper {
    /// Get color.
    ///
    /// `value` must be in the range [0.0, 1.0].
    pub fn color(&self, value: f64, mode: ColorMode) -> tui::style::Color {
        let rgb =
            HjColor(
                self.gradient
                    .eval_continuous(if self.invert { 1.0 - value } else { value }),
            );

        match mode {
            ColorMode::TrueColor => rgb.into(),
            ColorMode::Ansi256 => tui::style::Color::Indexed(ansi_colours::ansi256_from_rgb(rgb)),
        }
    }

    pub fn color_max_contrast(&self, value: f64, _mode: ColorMode) -> tui::style::Color {
        if HjColor(
            self.gradient
                .eval_continuous(if self.invert { 1.0 - value } else { value }),
        )
        .luma()
            < 128.0
        {
            tui::style::Color::White
        } else {
            tui::style::Color::Black
        }
    }
}

static GREYS: ColorMapper = ColorMapper {
    gradient: colorous::GREYS,
    invert: true,
};
static INFERNO: ColorMapper = ColorMapper {
    gradient: colorous::INFERNO,
    invert: false,
};
static TURBO: ColorMapper = ColorMapper {
    gradient: colorous::TURBO,
    invert: false,
};
static MAGMA: ColorMapper = ColorMapper {
    gradient: colorous::MAGMA,
    invert: false,
};
/*static PLASMA: ColorMapper = ColorMapper {
    gradient: colorous::PLASMA,
    invert: false,
};*/
static VIRIDIS: ColorMapper = ColorMapper {
    gradient: colorous::VIRIDIS,
    invert: false,
};
static CUBEHELIX: ColorMapper = ColorMapper {
    gradient: colorous::CUBEHELIX,
    invert: false,
};
static RAINBOW: ColorMapper = ColorMapper {
    gradient: colorous::SINEBOW,
    invert: false,
};

#[derive(Debug, Clone, Copy)]
pub enum ColorMap {
    Greys,
    Inferno,
    Turbo,
    Magma,
    //Plasma,
    Viridis,
    Cubehelix,
    Rainbow,
}

impl ColorMap {
    pub fn get(&self) -> &'static ColorMapper {
        match self {
            ColorMap::Greys => &GREYS,
            ColorMap::Inferno => &INFERNO,
            ColorMap::Turbo => &TURBO,
            ColorMap::Magma => &MAGMA,
            //ColorMap::Plasma => &PLASMA,
            ColorMap::Viridis => &VIRIDIS,
            ColorMap::Cubehelix => &CUBEHELIX,
            ColorMap::Rainbow => &RAINBOW,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ColorMap::Greys => ColorMap::Inferno,
            ColorMap::Inferno => ColorMap::Turbo,
            ColorMap::Turbo => ColorMap::Magma,
            ColorMap::Magma => /*ColorMap::Plasma,
            ColorMap::Plasma =>*/ ColorMap::Viridis,
            ColorMap::Viridis => ColorMap::Cubehelix,
            ColorMap::Cubehelix => ColorMap::Rainbow,
            ColorMap::Rainbow => ColorMap::Greys,
        }
    }
}

impl std::fmt::Display for ColorMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
