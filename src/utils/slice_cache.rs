use std::num::NonZeroUsize;

use crate::utils::sampling;
use noisy_float::prelude::*;

use super::brain_volume;

/*

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

fn _normalize_u8<D>(
    data: &ndarray::Array<f32, D>,
    intensity_range: (f32, f32),
) -> ndarray::Array<u8, D>
where
    D: ndarray::Dimension,
{
    let (imin, imax) = intensity_range;
    return (((data - imin) / (imax - imin)) * (u8::MAX as f32))
        .mapv(|v| num::clamp(v, u8::MIN as f32, u8::MAX as f32) as u8);
}

fn normalize_u16<D>(
    data: &ndarray::Array<f32, D>,
    intensity_range: (f32, f32),
) -> ndarray::Array<u16, D>
where
    D: ndarray::Dimension,
{
    let (imin, imax) = intensity_range;
    return (((data - imin) / (imax - imin)) * (u16::MAX as f32))
        .mapv(|v| num::clamp(v, u16::MIN as f32, u16::MAX as f32) as u16);
}

fn normalize_u16_f64<D>(
    data: &ndarray::Array<f64, D>,
    intensity_range: (f64, f64),
) -> ndarray::Array<u16, D>
where
    D: ndarray::Dimension,
{
    let (imin, imax) = intensity_range;
    return (((data - imin) / (imax - imin)) * (u16::MAX as f64))
        .mapv(|v| num::clamp(v, u16::MIN as f64, u16::MAX as f64) as u16);
}

fn make_image_gray<D>(data: ndarray::Array2<D>) -> image::ImageBuffer<Luma<D>, Vec<D>>
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

 */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CachableSlicerParams {
    pub axis: sampling::SliceAxis,
    pub depth: N64,
    pub default_value: N64,
    pub resolution: (usize, usize),
}

impl CachableSlicerParams {
    pub fn new(
        axis: sampling::SliceAxis,
        depth: N64,
        default_value: N64,
        resolution: (usize, usize),
    ) -> Self {
        Self {
            axis,
            depth,
            default_value,
            resolution,
        }
    }
}

#[derive(Debug)]
pub struct SliceCache {
    pub cache: lru::LruCache<CachableSlicerParams, ndarray::Array2<f64>>,
}

impl SliceCache {
    pub fn new() -> Self {
        Self {
            cache: lru::LruCache::new(NonZeroUsize::new(50).unwrap()),
        }
    }

    pub fn get(
        &mut self,
        volume: &brain_volume::BrainVolume,
        sample: CachableSlicerParams,
    ) -> ndarray::ArrayView2<f64> {
        self.cache
            .get_or_insert(sample, || {
                volume.world_slice(
                    sample.axis,
                    sample.depth.into(),
                    sample.default_value.into(),
                    sample.resolution,
                )

                /*let arr2d_norm: ndarray::Array2<u16> =
                        normalize_u16_f64(&arr2d, sample.intensity_range());
                let img: ImageRep = make_image_gray(arr2d_norm);*/
            })
            .view()
    }
}
