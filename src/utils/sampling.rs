use ndarray::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SliceAxis {
    X,
    Y,
    Z,
}

impl SliceAxis {
    pub fn from_index(index: usize) -> SliceAxis {
        match index {
            0 => SliceAxis::X,
            1 => SliceAxis::Y,
            2 => SliceAxis::Z,
            _ => panic!("Unsupported axis"),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            SliceAxis::X => 0,
            SliceAxis::Y => 1,
            SliceAxis::Z => 2,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SliceAxis::X => "X",
            SliceAxis::Y => "Y",
            SliceAxis::Z => "Z",
        }
    }
}

pub fn position_2d<T>(labels: &[T], axis: SliceAxis) -> (T, T, T)
where
    T: Copy,
{
    let index = labels[axis.index()];
    let y_index = labels[match axis {
        SliceAxis::X => 1,
        SliceAxis::Y => 0,
        SliceAxis::Z => 0,
    }];
    let x_index = labels[match axis {
        SliceAxis::X => 2,
        SliceAxis::Y => 2,
        SliceAxis::Z => 1,
    }];
    return (index, x_index, y_index);
}

#[derive(Debug, Clone, Copy)]
pub struct Cube {
    pub x0: f64,
    pub y0: f64,
    pub z0: f64,
    pub x1: f64,
    pub y1: f64,
    pub z1: f64,
    pub xd: f64,
    pub yd: f64,
    pub zd: f64,
}

impl Cube {
    pub fn new(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> Cube {
        let xd = x1 - x0;
        let yd = y1 - y0;
        let zd = z1 - z0;
        assert!(xd >= 0. && yd >= 0. && zd >= 0.);
        Cube {
            x0,
            y0,
            z0,
            x1,
            y1,
            z1,
            xd,
            yd,
            zd,
        }
    }

    pub fn corner_coords(&self) -> ndarray::Array2<f64> {
        ndarray::array![
            [self.x0, self.y0, self.z0, 1.],
            [self.x1, self.y1, self.z1, 1.],
            [self.x1, self.y0, self.z0, 1.],
            [self.x0, self.y1, self.z1, 1.],
            [self.x0, self.y0, self.z1, 1.],
            [self.x1, self.y1, self.z0, 1.],
            [self.x1, self.y0, self.z1, 1.],
            [self.x0, self.y1, self.z0, 1.],
        ].reversed_axes()
    }

    pub fn center(&self) -> ndarray::Array1<f64> {
        ndarray::array![
            (self.x0 + self.x1) / 2., (self.y0 + self.y1) / 2., (self.z0 + self.z1) / 2., 1.,
        ]
    }

    pub fn min(&self) -> ndarray::Array1<f64> {
        ndarray::array![self.x0, self.y0, self.z0, 1.]
    }

    pub fn max(&self) -> ndarray::Array1<f64> {
        ndarray::array![self.x1, self.y1, self.z1, 1.]
    }

    pub fn size(&self) -> ndarray::Array1<f64> {
        ndarray::array![
            self.xd,
            self.yd,
            self.zd,
        ]
    }
}

pub fn bounding_cube_from_shape_3d(shape: &[usize]) -> Cube {
    let (x, y, z) = ((shape[0] - 1) as f64, (shape[1] - 1) as f64, (shape[2] - 1) as f64);
    Cube::new(0., 0., 0., x, y, z)
}

pub fn bounding_cube_from_coords(coords: &ndarray::ArrayView2<f64>) -> Cube {
    let min = coords.map_axis(ndarray::Axis(1), |row| {
        row.iter().take(3).fold(f64::MAX, |acc, &x| acc.min(x))
    });
    let max = coords.map_axis(ndarray::Axis(1), |row| {
        row.iter().take(3).fold(f64::MIN, |acc, &x| acc.max(x))
    });
    Cube::new(min[0], min[1], min[2], max[0], max[1], max[2])
}

pub fn coords_apply_affine_transform(
    coords: &ndarray::ArrayView2<f64>,
    affine: &ndarray::ArrayView2<f64>,
) -> ndarray::Array2<f64> {
    affine.dot(coords)
}

pub fn invert_affine_transform(affine: &ndarray::ArrayView2<f64>) -> ndarray::Array2<f64> {
    let inv = nalgebra::Matrix4::from_iterator(affine.iter().cloned())
        .try_inverse()
        .unwrap();
    ndarray::array![
        [inv[(0, 0)], inv[(0, 1)], inv[(0, 2)], inv[(0, 3)]],
        [inv[(1, 0)], inv[(1, 1)], inv[(1, 2)], inv[(1, 3)]],
        [inv[(2, 0)], inv[(2, 1)], inv[(2, 2)], inv[(2, 3)]],
        [inv[(3, 0)], inv[(3, 1)], inv[(3, 2)], inv[(3, 3)]],
    ].reversed_axes()
}

pub fn slice_cube_3d_coords(
    axis: SliceAxis,
    out_width: usize,
    out_height: usize,
    out_depth: f64,
    cube: Cube,
) -> Array2<f64> {
    let (h_min, v_min, h_max, v_max) = match axis {
        SliceAxis::X => (cube.y0, cube.z0, cube.y1, cube.z1),
        SliceAxis::Y => (cube.x0, cube.z0, cube.x1, cube.z1),
        SliceAxis::Z => (cube.x0, cube.y0, cube.x1, cube.y1),
    };

    let mut coords = ndarray::Array2::<f64>::uninit((out_width * out_height, 4));
    let mut idx = 0;
    for v in 0..out_height {
        for h in 0..out_width {
            let xx = h_min + (h as f64 / (out_width - 1) as f64) * (h_max - h_min);
            let yy = v_min + (v as f64 / (out_height - 1) as f64) * (v_max - v_min);
            let (x, y, z) = match axis {
                SliceAxis::X => (
                    out_depth,
                    xx,
                    yy,
                ),
                SliceAxis::Y => (
                    xx,
                    out_depth,
                    yy,
                ),
                SliceAxis::Z => (
                    xx,
                    yy,
                    out_depth,
                ),
            };
            coords[[idx, 0]].write(x);
            coords[[idx, 1]].write(y);
            coords[[idx, 2]].write(z);
            coords[[idx, 3]].write(1.);
            idx += 1;
        }
    }
    unsafe { coords.assume_init().reversed_axes() }
}

pub fn map_coordinates_3d(
    input_array: &ArrayView3<f64>,
    coords: &ArrayView2<f64>,
    _default_value: f64,
) -> Array1<f64> {
    let coords = coords.reversed_axes();
    let shape = input_array.shape();
    let num_coords = coords.shape()[0];
    let mut output_array = Array1::<f64>::zeros(num_coords);

    for (idx, coord) in coords.axis_iter(ndarray::Axis(0)).enumerate() {
        let interpolated_value ;

        // Clamp coordinates to the array bounds
        let coord = ndarray::array![
            coord[0].clamp(0., shape[0] as f64 - 2.),
            coord[1].clamp(0., shape[1] as f64 - 2.),
            coord[2].clamp(0., shape[2] as f64 - 2.),
            1.
        ];

        // Use floor to determine the integer grid cell
        let (x, y, z) = (
            coord[0].floor() as usize,
            coord[1].floor() as usize,
            coord[2].floor() as usize,
        );

        // Calculate the fractional part of the coordinate
        let (dx, dy, dz) = (
            coord[0] - x as f64,
            coord[1] - y as f64,
            coord[2] - z as f64,
        );

        // Trilinear interpolation
        interpolated_value = (1.0 - dx) * (1.0 - dy) * (1.0 - dz) * input_array[[x, y, z]]
            + dx * (1.0 - dy) * (1.0 - dz) * input_array[[x + 1, y, z]]
            + (1.0 - dx) * dy * (1.0 - dz) * input_array[[x, y + 1, z]]
            + dx * dy * (1.0 - dz) * input_array[[x + 1, y + 1, z]]
            + (1.0 - dx) * (1.0 - dy) * dz * input_array[[x, y, z + 1]]
            + dx * (1.0 - dy) * dz * input_array[[x + 1, y, z + 1]]
            + (1.0 - dx) * dy * dz * input_array[[x, y + 1, z + 1]]
            + dx * dy * dz * input_array[[x + 1, y + 1, z + 1]];

        output_array[idx] = interpolated_value;
    }

    output_array
}


pub fn downsample_2d_array(
    input: &ArrayView2<f64>,
    factor: usize,
) -> Array2<f64> {
    let (rows, cols) = input.dim();
    
    let new_rows = rows / factor;
    let new_cols = cols / factor;
    
    let mut result = Array2::zeros((new_rows, new_cols));
    
    for i in 0..new_rows {
        for j in 0..new_cols {
            let start_row = i * factor;
            let start_col = j * factor;
            let end_row = start_row + factor;
            let end_col = start_col + factor;
            
            let chunk = input.slice(s![start_row..end_row, start_col..end_col]);
            let avg = chunk.sum() / (factor * factor) as f64;
            
            result[[i, j]] = avg;
        }
    }
    
    result
}