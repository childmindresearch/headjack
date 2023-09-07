use crate::sampler3d;

pub struct MetaDataWidget<'a> {
    pub image_sampler: &'a sampler3d::Sampler3D,
    pub entries: Vec<(String, String)>,
    pub max_key_len: usize,
}

impl<'a> MetaDataWidget<'a> {
    pub fn new(image_sampler: &'a sampler3d::Sampler3D) -> Self {
        let ndim = image_sampler.header.dim[0] as usize;
        let entries: Vec<(String, String)> = vec![
            (
                "Data type".to_owned(),
                format!("{:?}", image_sampler.header.data_type().unwrap()),
            ),
            (
                "Ndim".to_owned(),
                format!("{}", image_sampler.header.dim[0]),
            ),
            (
                "Shape".to_owned(),
                format!("{:?}", &image_sampler.header.dim[1..ndim + 1]),
            ),
            (
                "Units".to_owned(),
                format!(
                    "{:?} (space); {:?} (time)",
                    &image_sampler
                        .header
                        .xyzt_to_space()
                        .unwrap_or(nifti::Unit::Unknown),
                    &image_sampler
                        .header
                        .xyzt_to_time()
                        .unwrap_or(nifti::Unit::Unknown)
                ),
            ),
            (
                "Data scaling".to_owned(),
                format!(
                    "{} + {} * x",
                    image_sampler.header.scl_inter, image_sampler.header.scl_slope
                ),
            ),
            (
                "Display range".to_owned(),
                format!(
                    "[{}, {}]",
                    image_sampler.header.cal_min, image_sampler.header.cal_max
                ),
            ),
            (
                "Description".to_owned(),
                format!(
                    "'{}'",
                    String::from_utf8(image_sampler.header.descrip.clone())
                        .unwrap_or("<error>".to_owned())
                ),
            ),
            (
                "Intent".to_owned(),
                format!(
                    "'{}'",
                    String::from_utf8(image_sampler.header.intent_name.to_vec())
                        .unwrap_or("<error>".to_owned())
                ),
            ),
            (
                "Slice order".to_owned(),
                format!(
                    "{:?}",
                    &image_sampler
                        .header
                        .slice_order()
                        .unwrap_or(nifti::SliceOrder::Unknown)
                ),
            ),
            (
                "Slice duration".to_owned(),
                format!("{}", image_sampler.header.slice_duration),
            ),
            (
                "Affine".to_owned(),
                format!("{:?}", &image_sampler.header.srow_x),
            ),
            ("".to_owned(), format!("{:?}", &image_sampler.header.srow_y)),
            ("".to_owned(), format!("{:?}", &image_sampler.header.srow_z)),
        ];

        let max_key_len = entries.iter().map(|(k, _)| k.len()).max().unwrap();

        Self {
            image_sampler,
            entries,
            max_key_len,
        }
    }
}

impl tui::widgets::Widget for MetaDataWidget<'_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        for (i, (key, val)) in self.entries.iter().enumerate() {
            if i < (area.height - 1) as usize {
                buf.set_stringn(
                    area.x,
                    area.y + i as u16,
                    key,
                    self.max_key_len,
                    tui::style::Style::default()
                        .fg(tui::style::Color::Cyan)
                        .add_modifier(tui::style::Modifier::BOLD),
                );
                buf.set_stringn(
                    area.x + self.max_key_len as u16 + 1,
                    area.y + i as u16,
                    val,
                    area.width.into(),
                    tui::style::Style::default(),
                );
            } else if i == (area.height - 1) as usize {
                buf.set_stringn(
                    area.x + 1,
                    area.y + i as u16,
                    "[...]",
                    area.width.into(),
                    tui::style::Style::default(),
                );
            }
        }
    }
}

pub struct SliceWidget<'a> {
    pub intensity_range: (f32, f32),
    pub slice_position: Vec<usize>,
    pub axis: usize,
    pub image_sampler: &'a sampler3d::Sampler3D,
    pub image_cache: &'a mut sampler3d::ImageCache,
    pub block: Option<tui::widgets::Block<'a>>,
}

impl<'a> SliceWidget<'a> {
    pub fn new(
        image_sampler: &'a sampler3d::Sampler3D,
        image_cache: &'a mut sampler3d::ImageCache,
        intensity_range: (f32, f32),
        slice_position: Vec<usize>,
        axis: usize,
    ) -> SliceWidget<'a> {
        Self {
            intensity_range,
            slice_position,
            axis,
            image_sampler,
            image_cache,
            block: None,
        }
    }

    pub fn block(mut self, block: tui::widgets::Block<'a>) -> SliceWidget<'a> {
        self.block = Some(block);
        self
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

pub fn position_2d<T>(position: &Vec<T>, axis: usize) -> (&T, &T, &T) {
    let index = &position[axis];
    let y_index: &T = &position[match axis {
        0 => 1,
        1 => 0,
        2 => 0,
        _ => panic!("Unsupported axis"),
    }];
    let x_index: &T = &position[match axis {
        0 => 2,
        1 => 2,
        2 => 1,
        _ => panic!("Unsupported axis"),
    }];
    return (index, x_index, y_index);
}

lazy_static! {
    static ref RAS_LABELS: Vec<&'static str> = vec!["IS", "PA", "LR"];
}

impl tui::widgets::Widget for SliceWidget<'_> {
    fn render(mut self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let text_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let (index, x_index, y_index) = position_2d(&self.slice_position, self.axis);

        let sample = sampler3d::ImageSample::new(
            text_area.width,
            text_area.height * 2,
            *index,
            self.axis,
            self.intensity_range,
        );

        let (img_sized, img_sized_limits) = self.image_cache.get(&self.image_sampler.arr, sample);

        //let palette = vec![" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];

        assert!(img_sized.width() <= text_area.width as u32);
        assert!(img_sized.height() <= (text_area.height * 2) as u32);

        let x_offset = (text_area.width as u32 - img_sized.width()) / 2;
        let y_offset = ((text_area.height * 2) as u32 - img_sized.height()) / 2;

        //println!("x_offset: {}", x_offset);
        //println!("y_offset: {}", y_offset);

        let x_index_scaled =
            ((*x_index as f32 / img_sized_limits.0 as f32) * img_sized.width() as f32) as u32;
        let y_index_scaled =
            ((*y_index as f32 / img_sized_limits.1 as f32) * (img_sized.height()) as f32) as u32;

        // write img_sized into buf
        for y in (0..((img_sized.height() / 2) * 2)).step_by(2) {
            for x in 0..img_sized.width() {
                let (ix, iy): (u16, u16) = ((x + x_offset) as u16, ((y + y_offset) / 2) as u16);
                let val_upper = img_sized.get_pixel(x, y + 1)[0] as usize;
                let val_lower = img_sized.get_pixel(x, y)[0] as usize;

                let gradient = colorous::INFERNO;
                let col_upper = gradient.eval_rational(val_upper, u16::MAX as usize + 1);
                let col_lower = gradient.eval_rational(val_lower, u16::MAX as usize + 1);

                let c = buf.get_mut(text_area.left() + ix, text_area.bottom() - iy - 1);

                let symb =
                    tui::widgets::BorderType::line_symbols(tui::widgets::BorderType::Rounded);

                let crossair_y = y / 2 * 2 == y_index_scaled / 2 * 2;
                let crossair_x = x == x_index_scaled;

                if crossair_x || crossair_y {
                    let col_average =
                        gradient.eval_rational((val_lower + val_upper) / 2, u16::MAX as usize + 1);
                    let col_inv = colorous2tui(invert_color(col_average));
                    c.set_bg(colorous2tui(col_average));

                    match (crossair_x, crossair_y) {
                        (true, true) => c
                            .set_char(symb.cross.chars().next().unwrap())
                            .set_fg(col_inv),
                        (true, false) => c
                            .set_char(symb.vertical.chars().next().unwrap())
                            .set_fg(col_inv),
                        (false, true) => c
                            .set_char(symb.horizontal.chars().next().unwrap())
                            .set_fg(col_inv),
                        _ => panic!(),
                    };

                    if crossair_y && (x == 0) {
                        c.set_char(
                            position_2d::<&str>(&RAS_LABELS, self.axis)
                                .1
                                .chars()
                                .nth(0)
                                .unwrap(),
                        );
                    } else if crossair_x && (y == 0) {
                        c.set_char(
                            position_2d::<&str>(&RAS_LABELS, self.axis)
                                .2
                                .chars()
                                .nth(0)
                                .unwrap(),
                        );
                    } else if crossair_x && (y / 2 == img_sized.height() / 2 - 1) {
                        c.set_char(
                            position_2d::<&str>(&RAS_LABELS, self.axis)
                                .2
                                .chars()
                                .nth(1)
                                .unwrap(),
                        );
                    } else if crossair_y && (x == img_sized.width() - 1) {
                        c.set_char(
                            position_2d::<&str>(&RAS_LABELS, self.axis)
                                .1
                                .chars()
                                .nth(1)
                                .unwrap(),
                        );
                    }
                } else {
                    c.set_bg(colorous2tui(col_upper))
                        .set_char('▄')
                        .set_fg(colorous2tui(col_lower));
                };
            }
        }
    }
}
