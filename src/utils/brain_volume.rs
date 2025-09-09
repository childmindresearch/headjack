use std::error::Error;

use ndarray::prelude::*;
use nifti::{IntoNdArray, NiftiObject, NiftiHeader};
use crate::utils::sampling;

use crate::utils::argminmax2::MinMax2;

pub type BrainMetaData = nifti::header::NiftiHeader;

// local: array/data space
// world: scanner/transformed space

#[derive(Debug)]
pub struct BrainVolume {
    pub arr: ArrayD<f64>,
    pub local_bounds: sampling::Cube,
    pub world_bounds: sampling::Cube,
    pub intensity_range: (f64, f64),
    pub affine: ndarray::Array2<f64>,
    pub affine_inv: ndarray::Array2<f64>,
    pub header: BrainMetaData,
}

pub fn array_view_3d<'a>(arr: &'a ArrayD<f64>) -> ArrayView3<'a, f64> {
    let mut v = arr.view();
    for _ in 0..arr.ndim() - 3 {
        v = v.index_axis_move(ndarray::Axis(0), 0);
    }
    v.into_dimensionality().unwrap()
}


impl BrainVolume {
    pub fn from_nifti(path_nifti: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let nif = nifti::ReaderOptions::new().read_file(path_nifti)?;
        let header = nif.header().to_owned();
        let affine = header_get_affine(&header);
        let affine_inv = sampling::invert_affine_transform(&affine.view());

        let arr = { nif.into_volume().into_ndarray::<f64>()?.reversed_axes() };
        let arr_first_3d_slice = array_view_3d(&arr).to_owned();

        let local_bounds = sampling::bounding_cube_from_shape_3d(arr_first_3d_slice.shape());
        let local_array_corners = local_bounds.corner_coords();
        let world_array_corners = sampling::coords_apply_affine_transform(&local_array_corners.view(), &affine.view());
        let world_bounds = sampling::bounding_cube_from_coords(&world_array_corners.view());

        let intensity_range = arr_first_3d_slice.minmax2();

        Ok(Self {
            arr,
            local_bounds,
            world_bounds,
            intensity_range,
            affine,
            affine_inv,
            header,
        })
    }

    pub fn array_view_3d(&self) -> ArrayView3<f64> {
        array_view_3d(&self.arr)
    }

    pub fn world_slice(&self, axis: sampling::SliceAxis, depth: f64, default_value: f64, resolution: (usize, usize)) -> Array2<f64> {

        let world_sample_coords = sampling::slice_cube_3d_coords(axis, resolution.0, resolution.1, depth, self.world_bounds);
        let local_sample_coords = sampling::coords_apply_affine_transform(&world_sample_coords.view(), &self.affine_inv.view());

        let arr_view_3d = self.array_view_3d();
        
        let out_arr_flat = sampling::map_coordinates_3d(&arr_view_3d, &local_sample_coords.view(), default_value);
        let out_arr_2d = out_arr_flat.into_shape_with_order((resolution.1, resolution.0)).unwrap().reversed_axes();

        out_arr_2d
    }

    pub fn local_slice(&self, axis: sampling::SliceAxis, depth: f64, default_value: f64, resolution: (usize, usize)) -> Array2<f64> {

        let world_sample_coords = sampling::slice_cube_3d_coords(axis, resolution.0, resolution.1, depth, self.local_bounds);

        let arr_view_3d = self.array_view_3d();
        
        let out_arr_flat = sampling::map_coordinates_3d(&arr_view_3d, &world_sample_coords.view(), default_value);
        let out_arr_2d = out_arr_flat.into_shape_with_order(resolution).unwrap();

        out_arr_2d
    }
    
}

fn header_get_affine(header: &NiftiHeader) -> ndarray::Array2<f64> {
    let x = header.srow_x;
    let y = header.srow_y;
    let z = header.srow_z;
    ndarray::array![
        [x[0], x[1], x[2], x[3]],
        [y[0], y[1], y[2], y[3]],
        [z[0], z[1], z[2], z[3]],
        [0., 0., 0., 1.]
    ].mapv(|elem| elem as f64)
}

pub fn xyz_units_str(header: &BrainMetaData) -> &str {
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