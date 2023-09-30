use noisy_float::prelude::*;
use tui::{
    prelude::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
};

use crate::utils;

static COORDS: [&str; 3] = ["Superior", "Anterior", "Right"];

pub struct SliceParams {
    pub intensity_range: (f64, f64),
    pub position: Vec<f64>,

    pub color_mode: utils::colors::ColorMode,
    pub color_map: colorous::Gradient,
}

impl SliceParams {
    pub fn position_2d(&self, axis: usize) -> (f64, f64, f64) {
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
    pub volume: &'a utils::brain_volume::BrainVolume,
    pub image_cache: &'a mut utils::slice_cache::SliceCache,

    pub slice: &'a SliceParams,
    pub axis: usize,
    pub block: Option<tui::widgets::Block<'a>>,
}

impl<'a> SliceWidget<'a> {
    pub fn new(
        volume: &'a utils::brain_volume::BrainVolume,
        image_cache: &'a mut utils::slice_cache::SliceCache,
        slice: &'a SliceParams,
        axis: usize,
    ) -> SliceWidget<'a> {
        Self {
            volume,
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

fn fit_relative(src_x: f64, src_y: f64, dest_x: f64, dest_y: f64) -> (f64, f64) {
    let src_aspect = src_x as f32 / src_y as f32;
    let dest_aspect = dest_x as f32 / dest_y as f32;

    let resize_factor = if src_aspect >= dest_aspect {
        dest_x as f32 / src_x as f32
    } else {
        dest_y as f32 / src_y as f32
    };

    return (
        (src_x as f32 * resize_factor) as f64,
        (src_y as f32 * resize_factor) as f64,
    );
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

        let (world_v_min, world_v_max, world_h_min, world_h_max) = match self.axis {
            0 => (
                self.volume.world_bounds.y0,
                self.volume.world_bounds.y1,
                self.volume.world_bounds.z0,
                self.volume.world_bounds.z1,
            ),
            1 => (
                self.volume.world_bounds.x0,
                self.volume.world_bounds.x1,
                self.volume.world_bounds.z0,
                self.volume.world_bounds.z1,
            ),
            2 => (
                self.volume.world_bounds.x0,
                self.volume.world_bounds.x1,
                self.volume.world_bounds.y0,
                self.volume.world_bounds.y1,
            ),
            _ => (0.0, 0.0, 0.0, 0.0),
        };

        let screen_width: usize = text_area.width as usize;
        let screen_height: usize = text_area.height as usize * 2;

        let (sampling_width, sampling_height) = fit_relative(
            world_h_max - world_h_min, 
            world_v_max - world_v_min, 
            screen_width as f64, 
            screen_height as f64
        );

        let (index, x_index, y_index) = self.slice.position_2d(self.axis);

        let ssaa_factor = 2;

        let sample = utils::slice_cache::CachableSlicerParams::new(
            utils::sampling::SliceAxis::from_index(self.axis),
            n64(index),
            n64(0.),
            (sampling_height as usize * ssaa_factor, sampling_width as usize * ssaa_factor),
        );

        let img_arr = self.image_cache.get(&self.volume, sample).reversed_axes();

        let img_arr = utils::sampling::downsample_2d_array(&img_arr.view(), ssaa_factor);

        let img_arr_width = img_arr.shape()[0];
        let img_arr_height = img_arr.shape()[1];

        assert!(img_arr_width <= screen_width, "Error: img_arr_width: {} > screen_width: {}", img_arr_width, screen_width);
        assert!(img_arr_height <= screen_height, "Error: img_arr_height: {} > screen_height: {}", img_arr_height, screen_height);

        let x_offset = (screen_width - img_arr_width) / 2;
        let y_offset = (screen_height - img_arr_height) / 2;

        let intensity_r = self.slice.intensity_range.1 - self.slice.intensity_range.0;

        let y_index_rational = ((y_index - world_v_min) / (world_v_max - world_v_min) * (img_arr_height as f64)) as usize;
        let x_index_rational = ((x_index - world_h_min) / (world_h_max - world_h_min) * (img_arr_width as f64)) as usize;

        // write img_sized into buf
        for y in (0..((img_arr_height / 2) * 2)).step_by(2) {
            for x in 0..img_arr_width {
                let (ix, iy): (u16, u16) = ((x + x_offset) as u16, ((y + y_offset) / 2) as u16);
                let val_upper = (img_arr[[x as usize, y as usize + 1]]
                    - self.slice.intensity_range.0)
                    / intensity_r;
                let val_lower = (img_arr[[x as usize, y as usize]] - self.slice.intensity_range.0)
                    / intensity_r;

                let col_upper = utils::colors::calc_termcolor_continuous(
                    self.slice.color_mode,
                    self.slice.color_map,
                    val_upper,
                );
                let col_lower = utils::colors::calc_termcolor_continuous(
                    self.slice.color_mode,
                    self.slice.color_map,
                    val_lower,
                );

                let c = buf.get_mut(text_area.left() + ix, text_area.bottom() - iy - 1);

                let symb =
                    tui::widgets::BorderType::line_symbols(tui::widgets::BorderType::Rounded);

                let crossair_y = y / 2 * 2 == y_index_rational / 2 * 2;
                let crossair_x = x == x_index_rational;

                if crossair_x || crossair_y {
                    let col_average = utils::colors::calc_termcolor_continuous(
                        self.slice.color_mode,
                        self.slice.color_map,
                        (val_lower + val_upper) / 2.,
                    );
                    let col_inv = utils::colors::calc_termcolor_inverted_continuous(
                        self.slice.color_mode,
                        self.slice.color_map,
                        (val_lower + val_upper) / 2.,
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
                    } else if crossair_x && (y / 2 == img_arr_height / 2 - 1) {
                        c.set_char(
                            position_2d(&RAS_LABELS, self.axis)
                                .2
                                .chars()
                                .nth(1)
                                .unwrap(),
                        );
                    } else if crossair_y && (x == img_arr_width - 1) {
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

fn xyz_units_str(header: &utils::brain_volume::BrainMetaData) -> &str {
    match header.xyzt_to_space().unwrap_or(nifti::Unit::Unknown) {
        nifti::Unit::Unknown => "au",
        nifti::Unit::Meter => "m",
        nifti::Unit::Mm => "mm",
        nifti::Unit::Micron => "µm",
        nifti::Unit::Sec => "s",
        nifti::Unit::Msec => "ms",
        nifti::Unit::Usec => "µs",
        nifti::Unit::Hz => "Hz",
        nifti::Unit::Ppm => "ppm",
        nifti::Unit::Rads => "rad/s",
    }
}

pub struct XyzWidget<'a> {
    pub volume: &'a utils::brain_volume::BrainVolume,
    pub image_cache: &'a mut utils::slice_cache::SliceCache,

    pub slice: &'a SliceParams,
    pub block: Option<tui::widgets::Block<'a>>,
}

impl<'a> XyzWidget<'a> {
    pub fn new(
        volume: &'a utils::brain_volume::BrainVolume,
        image_cache: &'a mut utils::slice_cache::SliceCache,
        slice: &'a SliceParams,
    ) -> XyzWidget<'a> {
        Self {
            volume,
            image_cache,
            slice,
            block: None,
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

            SliceWidget::new(self.volume, self.image_cache, self.slice, display_axis)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(
                            "{} = {:.2} {}",
                            COORDS[display_axis],
                            self.slice.position[display_axis],
                            xyz_units_str(&self.volume.header)
                        ))
                        .border_type(BorderType::Rounded),
                )
                .render(main_layout[i], buf);
        }
    }
}
