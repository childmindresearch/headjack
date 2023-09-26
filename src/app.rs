use std::error;

use crate::{utils, widgets};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Xyz,
    MetaData,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub file_path: String,
    pub image_sampler: utils::sampler3d::Sampler3D,
    pub image_cache: utils::sampler3d::ImageCache,
    pub intensity_range: (f32, f32),
    pub slice_position: Vec<usize>,
    pub increment: usize,
    pub mode: AppMode,
    pub color_map: colorous::Gradient,
    pub color_mode: utils::colors::ColorMode,
    pub metadata: widgets::key_value_list_widget::KeyValueList,
    pub metadata_index: usize,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(file_path: &str, color_mode: utils::colors::ColorMode) -> Self {
        println!("Read nifti...");
        let sampler = utils::sampler3d::Sampler3D::from_nifti(file_path).unwrap();
        let intensity_range = sampler.intensity_range();
        let middle_slice = sampler.middle_slice();

        let increment = sampler.shape().iter().copied().sum::<usize>() / sampler.shape().len() / 32;

        let metadata = utils::metadata::make_metadata_key_value_list(&sampler);

        println!(" done!");
        //println!("Image dimensions {:?}", sampler.shape());

        Self {
            running: true,
            file_path: file_path.to_string(),
            image_sampler: sampler,
            image_cache: utils::sampler3d::ImageCache::new(),
            intensity_range: intensity_range,
            slice_position: middle_slice,
            increment: increment,
            mode: AppMode::Xyz,
            color_map: colorous::INFERNO,
            color_mode: color_mode,
            metadata: metadata,
            metadata_index: 0,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_slice(&mut self, axis: usize) {
        match axis {
            0..=2 => {
                let shape = self.image_sampler.shape();
                if self.slice_position[axis] + self.increment < shape[axis] {
                    self.slice_position[axis] += self.increment;
                }
            }
            _ => {}
        }
    }

    pub fn decrement_slice(&mut self, axis: usize) {
        match axis {
            0..=2 => {
                if self.slice_position[axis] >= self.increment {
                    self.slice_position[axis] -= self.increment;
                }
            }
            _ => {}
        }
    }

    pub fn toggle_tab(&mut self) {
        self.mode = match self.mode {
            AppMode::Xyz => AppMode::MetaData,
            AppMode::MetaData => AppMode::Xyz,
        }
    }
}
