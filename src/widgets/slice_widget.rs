use noisy_float::prelude::*;
use tui::{
    prelude::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
};

use crate::utils;

static COORDS: [&str; 3] = ["Superior", "Anterior", "Right"];
static RAS_LABELS: [[char; 2]; 3] = [['I', 'S'], ['P', 'A'], ['L', 'R']];

pub struct SliceParams {
    pub intensity_range: (f64, f64),
    pub position: Vec<f64>,

    pub color_mode: utils::colors::ColorMode,
    pub color_map: utils::colors::ColorMap,
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

pub trait DoubleYImage {
    /// Returns the size of the image in pixels. 
    /// This is done to assert (size().0 >= render area width && size().1 >= render area height * 2)
    fn size(&self) -> (usize, usize);

    /// Returns the character to be rendered at the given position, if any.
    /// 
    /// If there is a character to be rendered, instead of rendering 2 vertical pixels,
    /// the color returned by get_double() 
    /// will be used as the background color, and the color returned 
    /// by get_double_max_contrast() will be used as the foreground color.
    fn has_overlay(&self, x: usize, y: usize) -> Option<char>;

    /// Returns the color of the pixel at the given position.
    fn get(&self, x: usize, y: usize) -> tui::style::Color;

    /// Returns a contrasting color to the one returned by get().
    fn get_max_contrast(&self, x: usize, y: usize) -> tui::style::Color;

    /// Returns the average color of the double pixel at the given position ((x, y) + (x, y + 1)) / 2.
    fn get_double(&self, x: usize, y: usize) -> tui::style::Color;

    /// Returns a contrasting color to the one returned by get_double().
    fn get_double_max_contrast(&self, x: usize, y: usize) -> tui::style::Color;
}

/// Renders a DoubleYImage.
/// 
/// The image is rendered in the given area, which must be twice as tall as the image.
/// Special unicode character is used to render two colors for each terminal cell.
/// 
/// Overlay characters can be selectively rendered on top of the image.
/// In that case an average color is used as the background color, and a contrasting 
/// color is used as the foreground (overlay) color.
pub struct DoubleYImageRenderer<'a> {
    pub image: &'a dyn DoubleYImage,
}

impl<'a> DoubleYImageRenderer<'a> {
    pub fn new(image: &'a dyn DoubleYImage) -> DoubleYImageRenderer<'a> {
        Self { image }
    }
}

impl tui::widgets::Widget for DoubleYImageRenderer<'_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        assert!(
            self.image.size().0 >= area.width as usize,
            "Image width: {} != area width: {}",
            self.image.size().0,
            area.width
        );
        assert!(
            self.image.size().1 >= area.height as usize * 2,
            "Image height: {} != area height: {}",
            self.image.size().1,
            area.height * 2
        );

        for y in (0..(area.height as usize * 2)).step_by(2) {
            for x in 0..(area.width as usize) {
                let bx = area.left() + x as u16;
                let by = area.bottom() - (y / 2) as u16 - 1; // y is flipped
                let c = buf.get_mut(bx, by);

                match self.image.has_overlay(x, y) {
                    Some(overlay) => {
                        let color = self.image.get_double(x, y);
                        let color_inverted = self.image.get_double_max_contrast(x, y);
                        c.set_bg(color).set_fg(color_inverted).set_char(overlay);
                    }
                    None => {
                        let color_upper = self.image.get(x, y + 1);
                        let color_lower = self.image.get(x, y);

                        c.set_bg(color_upper).set_fg(color_lower).set_char('▄');
                    }
                }
            }
        }
    }
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

struct RenderedSlice<T: Fn(usize, usize) -> Option<char>> {
    pub image: ndarray::Array2<f64>,
    pub color_mapper: &'static utils::colors::ColorMapper,
    pub color_mode: utils::colors::ColorMode,
    pub has_overlay_callback: T,
}

impl<T: Fn(usize, usize) -> Option<char>> RenderedSlice<T> {
    pub fn new(
        slice: &SliceParams,
        image: ndarray::Array2<f64>,
        has_overlay_callback: T,
    ) -> RenderedSlice<T> {
        let intensity_r = slice.intensity_range.1 - slice.intensity_range.0;
        let image = image.mapv(|x| (x - slice.intensity_range.0) / intensity_r);
        let color_mapper = slice.color_map.get();

        Self {
            image,
            color_mapper,
            color_mode: slice.color_mode,
            has_overlay_callback,
        }
    }
}

impl<T: Fn(usize, usize) -> Option<char>> DoubleYImage for RenderedSlice<T> {
    fn size(&self) -> (usize, usize) {
        let shape = self.image.shape();
        (shape[0], shape[1])
    }

    fn has_overlay(&self, x: usize, y: usize) -> Option<char> {
        (self.has_overlay_callback)(x, y)
    }

    fn get(&self, x: usize, y: usize) -> tui::style::Color {
        let val = self.image[[x, y]];
        self.color_mapper.color(val, self.color_mode)
    }

    fn get_max_contrast(&self, x: usize, y: usize) -> tui::style::Color {
        let val = self.image[[x, y]];
        self.color_mapper.color_max_contrast(val, self.color_mode)
    }

    fn get_double(&self, x: usize, y: usize) -> tui::style::Color {
        let val_lower = self.image[[x, y]];
        let val_upper = self.image[[x, y + 1]];
        let val_average = (val_lower + val_upper) / 2.;
        self.color_mapper.color(val_average, self.color_mode)
    }

    fn get_double_max_contrast(&self, x: usize, y: usize) -> tui::style::Color {
        let val_lower = self.image[[x, y]];
        let val_upper = self.image[[x, y + 1]];
        let val_average = (val_lower + val_upper) / 2.;
        self.color_mapper
            .color_max_contrast(val_average, self.color_mode)
    }
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

        let (world_v_min, world_v, world_h_min, world_h) = match self.axis {
            0 => (
                self.volume.world_bounds.y0,
                self.volume.world_bounds.yd,
                self.volume.world_bounds.z0,
                self.volume.world_bounds.zd,
            ),
            1 => (
                self.volume.world_bounds.x0,
                self.volume.world_bounds.xd,
                self.volume.world_bounds.z0,
                self.volume.world_bounds.zd,
            ),
            2 => (
                self.volume.world_bounds.x0,
                self.volume.world_bounds.xd,
                self.volume.world_bounds.y0,
                self.volume.world_bounds.yd,
            ),
            _ => (0.0, 0.0, 0.0, 0.0),
        };

        let screen_width: usize = text_area.width as usize;
        let screen_height: usize = text_area.height as usize * 2;

        let (sampling_width, sampling_height) =
            fit_relative(world_h, world_v, screen_width as f64, screen_height as f64);

        let (index, x_index, y_index) = self.slice.position_2d(self.axis);

        let sampling_max = std::cmp::min(sampling_width as usize, sampling_height as usize);
        let data_max = std::cmp::max(
            self.volume.local_bounds.xd as usize,
            self.volume.local_bounds.yd as usize,
        );
        let downscale_factor = (data_max as f64 / sampling_max as f64).ceil() as usize;
        let ssaa_factor = std::cmp::min(std::cmp::max(1, downscale_factor), 16);

        let sample = utils::slice_cache::CachableSlicerParams::new(
            utils::sampling::SliceAxis::from_index(self.axis),
            n64(index),
            n64(0.),
            (
                sampling_height as usize * ssaa_factor,
                sampling_width as usize * ssaa_factor,
            ),
        );

        let img_arr = self.image_cache.get(&self.volume, sample).reversed_axes();

        let img_arr = utils::sampling::downsample_2d_array(&img_arr.view(), ssaa_factor);

        let img_arr_width = img_arr.shape()[0];
        let img_arr_height = img_arr.shape()[1];

        assert!(
            img_arr_width <= screen_width,
            "Error: img_arr_width: {} > screen_width: {}",
            img_arr_width,
            screen_width
        );
        assert!(
            img_arr_height <= screen_height,
            "Error: img_arr_height: {} > screen_height: {}",
            img_arr_height,
            screen_height
        );

        let x_offset = (screen_width - img_arr_width) / 2;
        let y_offset = (screen_height - img_arr_height) / 2;

        let image_render_area = tui::layout::Rect::new(
            text_area.left() + x_offset as u16,
            text_area.top() + (y_offset / 2) as u16,
            img_arr_width as u16,
            (img_arr_height / 2) as u16,
        );

        let y_index_rational =
            ((y_index - world_v_min) / world_v * (img_arr_height as f64)) as usize;
        let x_index_rational =
            ((x_index - world_h_min) / world_h * (img_arr_width as f64)) as usize;

        let symb = tui::widgets::BorderType::line_symbols(tui::widgets::BorderType::Rounded);
        let symb_cross = symb.cross.chars().next().unwrap();
        let symb_vertical = symb.vertical.chars().next().unwrap();
        let symb_horizontal = symb.horizontal.chars().next().unwrap();

        let img_slice = RenderedSlice::new(self.slice, img_arr, |x, y| {
            let crossair_y = y / 2 * 2 == y_index_rational / 2 * 2;
            let crossair_x = x == x_index_rational;

            match (crossair_x, crossair_y) {
                (true, true) => Some(symb_cross),
                (true, false) => {
                    if y == 0 {
                        Some(position_2d(&RAS_LABELS, self.axis).2[0])
                    } else if y / 2 == img_arr_height / 2 - 1 {
                        Some(position_2d(&RAS_LABELS, self.axis).2[1])
                    } else {
                        Some(symb_vertical)
                    }
                }
                (false, true) => {
                    if x == 0 {
                        Some(position_2d(&RAS_LABELS, self.axis).1[0])
                    } else if x == img_arr_width - 1 {
                        Some(position_2d(&RAS_LABELS, self.axis).1[1])
                    } else {
                        Some(symb_horizontal)
                    }
                }
                (false, false) => None,
            }
        });

        DoubleYImageRenderer::new(&img_slice).render(image_render_area, buf);
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
                            utils::brain_volume::xyz_units_str(&self.volume.header)
                        ))
                        .border_type(BorderType::Rounded),
                )
                .render(main_layout[i], buf);
        }
    }
}
