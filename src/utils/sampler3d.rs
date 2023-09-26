use std::{error::Error, num::NonZeroUsize};

use image::Luma;
use ndarray::{s, Array, Array2, ArrayD, ArrayView2, Ix2};
use nifti::{IntoNdArray, NiftiObject};
use noisy_float::types::{n32, N32};

use crate::utils::argminmax2::MinMax2;

pub type BrainVolume = ArrayD<f32>;
pub type BrainMetaData = nifti::header::NiftiHeader;

/// Scale x and y relative so that the bigger one will be max_val.
fn _scale_relative(x: u32, y: u32, max_val: u32) -> (u32, u32) {
    let (fx, fy, fmax_val) = (x as f32, y as f32, max_val as f32);

    let fxy_max = std::cmp::max(x, y) as f32;

    let out_x = std::cmp::min(((fx / fxy_max) * fmax_val) as u32, max_val);
    let out_y = std::cmp::min(((fy / fxy_max) * fmax_val) as u32, max_val);

    return (out_x, out_y);
}

fn fit_relative(src_x: u32, src_y: u32, dest_x: u32, dest_y: u32) -> (u32, u32) {
    let src_aspect = src_x as f32 / src_y as f32;
    let dest_aspect = dest_x as f32 / dest_y as f32;

    let resize_factor = if src_aspect >= dest_aspect {
        dest_x as f32 / src_x as f32
    } else {
        dest_y as f32 / src_y as f32
    };

    return (
        (src_x as f32 * resize_factor) as u32,
        (src_y as f32 * resize_factor) as u32,
    );
}

fn _normalize_u8<D>(data: &Array<f32, D>, intensity_range: (f32, f32)) -> Array<u8, D>
where
    D: ndarray::Dimension,
{
    let (imin, imax) = intensity_range;
    return (((data - imin) / (imax - imin)) * (u8::MAX as f32))
        .mapv(|v| num::clamp(v, u8::MIN as f32, u8::MAX as f32) as u8);
}

fn normalize_u16<D>(data: &Array<f32, D>, intensity_range: (f32, f32)) -> Array<u16, D>
where
    D: ndarray::Dimension,
{
    let (imin, imax) = intensity_range;
    return (((data - imin) / (imax - imin)) * (u16::MAX as f32))
        .mapv(|v| num::clamp(v, u16::MIN as f32, u16::MAX as f32) as u16);
}

fn make_image_gray<D>(data: Array2<D>) -> image::ImageBuffer<Luma<D>, Vec<D>>
where
    D: image::Primitive,
{
    let width = data.shape()[0] as u32;
    let height = data.shape()[1] as u32;
    return image::ImageBuffer::<Luma<D>, Vec<D>>::from_raw(
        height,
        width,
        data.reversed_axes().into_raw_vec(),
    )
    .unwrap();
}

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

#[derive(Debug)]
pub struct Sampler3D {
    pub arr: BrainVolume,
    pub header: BrainMetaData,
}

impl Sampler3D {
    pub fn from_nifti(path_nifti: &str) -> Result<Self, Box<dyn Error>> {
        let nif = nifti::ReaderOptions::new().read_file(path_nifti)?;
        let header = nif.header().to_owned();

        let arr = { nif.into_volume().into_ndarray::<f32>()?.reversed_axes() };
        Ok(Self {
            arr: arr,
            header: header,
        })
    }

    pub fn intensity_range(&self) -> (f32, f32) {
        let mm = self.arr.minmax2();
        //let range = mm.1 - mm.0;
        return (mm.0, mm.1); // - range * 0.8
    }

    pub fn shape(&self) -> &[usize] {
        match self.arr.ndim() {
            3 => self.arr.shape(),
            4 => &self.arr.shape()[1..],
            _ => panic!(),
        }
    }

    pub fn middle_slice(&self) -> Vec<usize> {
        Vec::from(self.shape()).iter().map(|x| x / 2).collect()
    }
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

pub type ImageRep = image::ImageBuffer<Luma<u16>, Vec<u16>>;

#[derive(Debug)]
pub struct ImageCache {
    pub image_cache: lru::LruCache<ImageSample, (ImageRep, (usize, usize))>,
}

impl ImageCache {
    pub fn new() -> ImageCache {
        Self {
            image_cache: lru::LruCache::new(NonZeroUsize::new(50).unwrap()),
        }
    }

    pub fn get(&mut self, data: &BrainVolume, sample: ImageSample) -> &(ImageRep, (usize, usize)) {
        self.image_cache.get_or_insert(sample, || {
            let arr2d: ArrayView2<f32> = match data.ndim() {
                2 => data.view().into_dimensionality::<Ix2>().unwrap(),
                3 => {
                    let index_clamped = num::clamp(sample.index, 0, data.dim()[sample.axis] - 1);
                    let slice = match sample.axis {
                        0 => s![index_clamped, .., ..],
                        1 => s![.., index_clamped, ..],
                        2 => s![.., .., index_clamped],
                        _ => panic!("Unsupported axis"),
                    };
                    data.slice(slice).into_dimensionality::<Ix2>().unwrap()
                }
                4 => {
                    let index_clamped =
                        num::clamp(sample.index, 0, data.dim()[sample.axis + 1] - 1);
                    let slice = match sample.axis {
                        0 => s![0, index_clamped, .., ..],
                        1 => s![0, .., index_clamped, ..],
                        2 => s![0, .., .., index_clamped],
                        _ => panic!("Unsupported axis"),
                    };
                    data.slice(slice).into_dimensionality::<Ix2>().unwrap()
                }
                _ => panic!("Unsupported dimensionality"),
            };

            let image_limits = (arr2d.shape()[1], arr2d.shape()[0]);

            let arr2d_norm: Array2<u16> =
                normalize_u16(&arr2d.to_owned(), sample.intensity_range());
            let img: ImageRep = make_image_gray(arr2d_norm);

            let (target_width, target_height) = fit_relative(
                img.width(),
                img.height(),
                sample.width as u32,
                sample.height as u32,
            );

            (
                image::imageops::resize(
                    &img,
                    target_width,
                    target_height,
                    image::imageops::FilterType::Triangle,
                ),
                image_limits,
            )
        })
    }
}
