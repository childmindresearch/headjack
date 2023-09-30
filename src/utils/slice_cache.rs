use std::num::NonZeroUsize;

use crate::utils::sampling;
use noisy_float::prelude::*;

use super::brain_volume;

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
