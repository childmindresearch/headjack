use lru::LruCache;
use ndarray_stats::QuantileExt;
use std::{error::Error, num::NonZeroUsize, primitive};

use argminmax::ArgMinMax;
use image::GrayImage;
use ndarray::{s, Array, Array2, ArrayBase, ArrayD, ArrayView, ArrayView2, Ix2};
use nifti::{IntoNdArray, NiftiObject, NiftiVolume, ReaderStreamedOptions};
use noisy_float::types::{n32, n64, N32};

use crate::argminmax2::MinMax2;

/// Scale x and y relative so that the bigger one will be max_val.
fn scale_relative(x: u32, y: u32, max_val: u32) -> (u32, u32) {
    let (fx, fy, fmax_val) = (x as f32, y as f32, max_val as f32);

    let fxy_max = std::cmp::max(x, y) as f32;

    let out_x = std::cmp::min(((fx / fxy_max) * fmax_val) as u32, max_val);
    let out_y = std::cmp::min(((fy / fxy_max) * fmax_val) as u32, max_val);

    return (out_x, out_y);
}

fn normalize_u8<D>(data: &Array<f32, D>, intensity_range: (f32, f32)) -> Array<u8, D>
where
    D: ndarray::Dimension,
{
    let (imin, imax) = intensity_range;
    return (((data - imin) / (imax - imin)) * 255.).mapv(|v| num::clamp(v, 0., 255.) as u8);
}

fn make_image_gray(data: Array2<u8>) -> GrayImage {
    let width = data.shape()[0] as u32;
    let height = data.shape()[1] as u32;
    return GrayImage::from_raw(height, width, data.reversed_axes().into_raw_vec()).unwrap();
}

pub type BrainVolume = ArrayD<f32>;

pub fn read_nifti(path_nifti: &str) -> Result<BrainVolume, Box<dyn Error>> {
    println!("Read file...");

    let re = {
        nifti::ReaderOptions::new()
            .read_file(path_nifti)?
            .into_volume()
            .into_ndarray::<f32>()?
            .reversed_axes()
    };

    println!("Done!");

    Ok(re)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSample {
    pub width: u16,
    pub height: u16,
    pub index: usize,
    pub axis: usize,
    pub intensity_min: N32,
    pub intensity_max: N32,
}

impl ImageSample {
    pub fn new(
        width: u16,
        height: u16,
        index: usize,
        axis: usize,
        intensity_range: (f32, f32),
    ) -> Self {
        Self {
            width,
            height,
            index,
            axis,
            intensity_min: n32(intensity_range.0),
            intensity_max: n32(intensity_range.1),
        }
    }

    pub fn intensity_range(&self) -> (f32, f32) {
        (self.intensity_min.into(), self.intensity_max.into())
    }
}

#[derive(Debug)]
pub struct ImageCache {
    pub image_cache: LruCache<ImageSample, (GrayImage, (usize, usize))>,
    pub data: BrainVolume,
}

impl ImageCache {
    pub fn new(data: BrainVolume) -> ImageCache {
        Self {
            image_cache: LruCache::new(NonZeroUsize::new(50).unwrap()),
            data,
        }
    }

    pub fn get(&mut self, sample: ImageSample) -> &(GrayImage, (usize, usize)) {
        self.image_cache.get_or_insert(sample, || {
            let arr2d: ArrayView2<f32> = match self.data.ndim() {
                2 => self.data.view().into_dimensionality::<Ix2>().unwrap(),
                3 => {
                    let index_clamped =
                        num::clamp(sample.index, 0, self.data.dim()[sample.axis] - 1);
                    let slice = match sample.axis {
                        0 => s![index_clamped, .., ..],
                        1 => s![.., index_clamped, ..],
                        2 => s![.., .., index_clamped],
                        _ => panic!("Unsupported axis"),
                    };
                    self.data.slice(slice).into_dimensionality::<Ix2>().unwrap()
                }
                _ => panic!("Unsupported dimensionality"),
            };

            let image_limits = (arr2d.shape()[0], arr2d.shape()[1]);

            let arr2d_norm: Array2<u8> = normalize_u8(&arr2d.to_owned(), sample.intensity_range());
            let img: GrayImage = make_image_gray(arr2d_norm);

            (image::imageops::resize(
                &img,
                sample.width.into(),
                sample.height.into(),
                image::imageops::FilterType::Triangle,
            ),image_limits)
        })
    }
}

pub struct SliceWidget<'a> {
    pub intensity_range: (f32, f32),
    pub slice_position: Vec<usize>,
    pub axis: usize,
    pub image_cache: &'a mut ImageCache,
    pub block: Option<tui::widgets::Block<'a>>,
}

impl<'a> SliceWidget<'a> {
    pub fn new(
        image_cache: &'a mut ImageCache,
        intensity_range: (f32, f32),
        slice_position: Vec<usize>,
        axis: usize,
    ) -> SliceWidget {
        Self {
            intensity_range,
            slice_position,
            axis,
            image_cache,
            block: None,
        }
    }

    pub fn block(mut self, block: tui::widgets::Block<'a>) -> SliceWidget<'a> {
        self.block = Some(block);
        self
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

        let index = self.slice_position[self.axis];
        let x_index: usize = self.slice_position[match self.axis {
            0 => 1,
            1 => 0,
            2 => 0,
            _ => panic!("Unsupported axis"),
        }];
        let y_index: usize = self.slice_position[match self.axis {
            0 => 2,
            1 => 2,
            2 => 1,
            _ => panic!("Unsupported axis"),
        }];

        let sample = ImageSample::new(
            text_area.width,
            text_area.height,
            index,
            self.axis,
            self.intensity_range,
        );

        let (img_sized, img_sized_limits) = self.image_cache.get(sample);

        //let palette = vec![" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];

        assert!(img_sized.width() == text_area.width as u32);
        assert!(img_sized.height() == text_area.height as u32);

        let x_index_scaled =
            ((x_index as f32 / img_sized_limits.0 as f32) * img_sized.width() as f32) as u32;
        let y_index_scaled =
            ((y_index as f32 / img_sized_limits.1 as f32) * img_sized.height() as f32) as u32;

        // write img_sized into buf
        for y in 0..img_sized.height() {
            for x in 0..img_sized.width() {
                let (ix, iy): (u16, u16) = (x as u16, y as u16);
                let val = img_sized.get_pixel(x, y)[0];
                //let relative_val = val as f32 / 255.;
                //let idx = num::clamp((relative_val * 9.) as usize, 0, 9);

                //buf.get_mut(text_area.left() + ix, text_area.top() + iy).set_char(palette[idx].chars().next().unwrap());

                let c = buf
                    .get_mut(text_area.left() + ix, text_area.top() + iy)
                    .set_bg(tui::style::Color::Rgb(val, val, val));

                if y == y_index_scaled && x == x_index_scaled {
                    c.set_char('X');
                } else if y == y_index_scaled {
                    c.set_char('-');
                } else if x == x_index_scaled {
                    c.set_char('|');
                }
            }
        }
    }
}
