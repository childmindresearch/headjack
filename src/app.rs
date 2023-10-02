use crate::utils::argminmax2::MinMax2;
use crate::{utils, widgets};
use std::error::{self};

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

    pub verbose: bool,
    pub file_path: String,
    pub volume: utils::brain_volume::BrainVolume,
    pub image_cache: utils::slice_cache::SliceCache,
    pub intensity_range: (f64, f64),
    pub slice_position: Vec<f64>,
    pub increment: f64,
    pub mode: AppMode,
    pub color_map: utils::colors::ColorMap,
    pub color_mode: utils::colors::ColorMode,
    pub metadata: widgets::key_value_list_widget::KeyValueList,
    pub metadata_index: usize,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(
        verbose: bool,
        file_path: &str,
        color_mode: utils::colors::ColorMode,
    ) -> std::result::Result<Self, Box<dyn error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let volume = utils::brain_volume::BrainVolume::from_nifti(file_path)?;
        let intensity_range = volume.intensity_range;
        let middle_slice = volume.world_bounds.center().into_iter().collect();
        let increment = volume.world_bounds.size().minmax2().0 / 32.0;
        let metadata = utils::metadata::make_metadata_key_value_list(&volume.header);
        let duration = start.elapsed();

        // Guess whether image is a mask or not
        let color_map = if utils::metadata::nifti_type_is_integer(
            volume.header.data_type().unwrap_or(nifti::NiftiType::Uint8),
        ) && intensity_range.0 < 1.0e-7
            && (intensity_range.1 - 1.0).abs() < 1.0e-7
        {
            utils::colors::ColorMap::Greys
        } else {
            utils::colors::ColorMap::Inferno
        };

        if verbose {
            println!("Data loaded in: {:?}", duration);
        }

        Ok(Self {
            verbose,
            running: true,
            file_path: file_path.to_string(),
            volume: volume,
            image_cache: utils::slice_cache::SliceCache::new(),
            intensity_range: intensity_range,
            slice_position: middle_slice,
            increment: increment,
            mode: AppMode::Xyz,
            color_map: color_map,
            color_mode: color_mode,
            metadata: metadata,
            metadata_index: 0,
        })
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_slice(&mut self, axis: usize) {
        let (_, max) = match axis {
            0 => (self.volume.world_bounds.x0, self.volume.world_bounds.x1),
            1 => (self.volume.world_bounds.y0, self.volume.world_bounds.y1),
            2 => (self.volume.world_bounds.z0, self.volume.world_bounds.z1),
            _ => (0.0, 0.0),
        };

        match axis {
            0..=2 => {
                if self.slice_position[axis] <= max - self.increment {
                    self.slice_position[axis] += self.increment;
                }
            }
            _ => {}
        }
    }

    pub fn decrement_slice(&mut self, axis: usize) {
        let (min, _) = match axis {
            0 => (self.volume.world_bounds.x0, self.volume.world_bounds.x1),
            1 => (self.volume.world_bounds.y0, self.volume.world_bounds.y1),
            2 => (self.volume.world_bounds.z0, self.volume.world_bounds.z1),
            _ => (0.0, 0.0),
        };

        match axis {
            0..=2 => {
                if self.slice_position[axis] >= min + self.increment {
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

    pub fn toggle_color_map(&mut self) {
        self.color_map = self.color_map.next();
    }

    pub fn increment_metadata_index(&mut self) {
        if self.metadata_index < self.metadata.len() - 2 {
            self.metadata_index += 1;
        }
    }

    pub fn decrement_metadata_index(&mut self) {
        if self.metadata_index > 0 {
            self.metadata_index -= 1;
        }
    }
}
