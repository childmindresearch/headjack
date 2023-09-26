use tui::{prelude::{Constraint, Direction, Layout}, widgets::{Block, Borders, BorderType}};

use crate::utils;

static COORDS: [&str; 3] = ["Superior", "Anterior", "Right"];

pub struct SliceParams {
    pub intensity_range: (f32, f32),
    pub position: Vec<usize>,

    pub color_mode: utils::colors::ColorMode,
    pub color_map: colorous::Gradient,
}

impl SliceParams {
    pub fn position_2d(&self, axis: usize) -> (usize, usize, usize) {
        return position_2d(&self.position, axis);
    }
}

fn position_2d<T>(labels: &[T], axis: usize) -> (T, T, T)
where
    T: Copy,
{
    let index = labels[axis];
    let y_index = labels[match axis {
        0 => 1,
        1 => 0,
        2 => 0,
        _ => panic!("Unsupported axis"),
    }];
    let x_index = labels[match axis {
        0 => 2,
        1 => 2,
        2 => 1,
        _ => panic!("Unsupported axis"),
    }];
    return (index, x_index, y_index);
}

pub struct SliceWidget<'a> {
    pub image_sampler: &'a utils::sampler3d::Sampler3D,
    pub image_cache: &'a mut utils::sampler3d::ImageCache,

    pub slice: &'a SliceParams,
    pub axis: usize,
    pub block: Option<tui::widgets::Block<'a>>,
}

impl<'a> SliceWidget<'a> {
    pub fn new(
        image_sampler: &'a utils::sampler3d::Sampler3D,
        image_cache: &'a mut utils::sampler3d::ImageCache,
        slice: &'a SliceParams, 
        axis: usize) -> SliceWidget<'a> 
    {
        Self {
            image_sampler,
            image_cache,
            slice,
            axis,
            block: None,
        }
    }

    pub fn block(mut self, block: tui::widgets::Block<'a>) -> SliceWidget<'a> {
        self.block = Some(block);
        self
    }
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

        let (index, x_index, y_index) = self.slice.position_2d(self.axis);

        let sample = utils::sampler3d::ImageSample::new(
            text_area.width,
            text_area.height * 2,
            index,
            self.axis,
            self.slice.intensity_range,
        );

        let (img_sized, img_sized_limits) = self
            .image_cache
            .get(&self.image_sampler.arr, sample);

        assert!(img_sized.width() <= text_area.width as u32);
        assert!(img_sized.height() <= (text_area.height * 2) as u32);

        let x_offset = (text_area.width as u32 - img_sized.width()) / 2;
        let y_offset = ((text_area.height * 2) as u32 - img_sized.height()) / 2;

        let x_index_scaled =
            ((x_index as f32 / img_sized_limits.0 as f32) * img_sized.width() as f32) as u32;
        let y_index_scaled =
            ((y_index as f32 / img_sized_limits.1 as f32) * (img_sized.height()) as f32) as u32;

        // write img_sized into buf
        for y in (0..((img_sized.height() / 2) * 2)).step_by(2) {
            for x in 0..img_sized.width() {
                let (ix, iy): (u16, u16) = ((x + x_offset) as u16, ((y + y_offset) / 2) as u16);
                let val_upper = img_sized.get_pixel(x, y + 1)[0] as usize;
                let val_lower = img_sized.get_pixel(x, y)[0] as usize;

                let col_upper = utils::colors::calc_termcolor(
                    self.slice.color_mode,
                    self.slice.color_map,
                    val_upper,
                    u16::MAX as usize + 1,
                );
                let col_lower = utils::colors::calc_termcolor(
                    self.slice.color_mode,
                    self.slice.color_map,
                    val_lower,
                    u16::MAX as usize + 1,
                );

                let c = buf.get_mut(text_area.left() + ix, text_area.bottom() - iy - 1);

                let symb =
                    tui::widgets::BorderType::line_symbols(tui::widgets::BorderType::Rounded);

                let crossair_y = y / 2 * 2 == y_index_scaled / 2 * 2;
                let crossair_x = x == x_index_scaled;

                if crossair_x || crossair_y {
                    let col_average = utils::colors::calc_termcolor(
                        self.slice.color_mode,
                        self.slice.color_map,
                        (val_lower + val_upper) / 2,
                        u16::MAX as usize + 1,
                    );
                    let col_inv = utils::colors::calc_termcolor_inverted(
                        self.slice.color_mode,
                        self.slice.color_map,
                        (val_lower + val_upper) / 2,
                        u16::MAX as usize + 1,
                    );
                    c.set_bg(col_average);

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
                            position_2d(&RAS_LABELS, self.axis)
                                .1
                                .chars()
                                .nth(0)
                                .unwrap(),
                        );
                    } else if crossair_x && (y == 0) {
                        c.set_char(
                            position_2d(&RAS_LABELS, self.axis)
                                .2
                                .chars()
                                .nth(0)
                                .unwrap(),
                        );
                    } else if crossair_x && (y / 2 == img_sized.height() / 2 - 1) {
                        c.set_char(
                            position_2d(&RAS_LABELS, self.axis)
                                .2
                                .chars()
                                .nth(1)
                                .unwrap(),
                        );
                    } else if crossair_y && (x == img_sized.width() - 1) {
                        c.set_char(
                            position_2d(&RAS_LABELS, self.axis)
                                .1
                                .chars()
                                .nth(1)
                                .unwrap(),
                        );
                    }
                } else {
                    c.set_bg(col_upper).set_char('▄').set_fg(col_lower);
                };
            }
        }
    }
}

pub struct XyzWidget<'a> {
    pub image_sampler: &'a utils::sampler3d::Sampler3D,
    pub image_cache: &'a mut utils::sampler3d::ImageCache,

    pub slice: &'a SliceParams,
    pub block: Option<tui::widgets::Block<'a>>,
}

impl<'a> XyzWidget<'a> {
    pub fn new(
        image_sampler: &'a utils::sampler3d::Sampler3D,
        image_cache: &'a mut utils::sampler3d::ImageCache,
        slice: &'a SliceParams
    ) -> XyzWidget<'a> {
        Self { 
            image_sampler,
            image_cache,
            slice, 
            block: None 
        }
    }

    pub fn block(mut self, block: tui::widgets::Block<'a>) -> XyzWidget<'a> {
        self.block = Some(block);
        self
    }
}

impl<'a> tui::widgets::Widget for XyzWidget<'a> {
    fn render(self, area: tui::prelude::Rect, buf: &mut tui::prelude::Buffer) {
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
            .split(area);

        for i in 0..3 {
            let display_axis = i;

            SliceWidget::new(
                self.image_sampler,
                self.image_cache,
                self.slice, 
                display_axis
            ).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(
                            "{} = {}",
                            COORDS[display_axis], self.slice.position[display_axis]
                        ))
                        .border_type(BorderType::Rounded),
                ).render(main_layout[i], buf);
        }
    }
}
