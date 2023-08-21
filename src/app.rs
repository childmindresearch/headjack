use std::error;

use crate::{brain, argminmax2::MinMax2};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub file_path: String,
    pub image_cache: brain::ImageCache,
    pub intensity_range: (f32, f32),
    pub slice_position: Vec<usize>,
    pub increment: usize,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(file_path: &str) -> Self {
        println!("Read nifti...");
        let nifti = brain::read_nifti(file_path).unwrap();
        let mm = nifti.minmax2();
        let range = mm.1 - mm.0;
        let mm = (mm.0, mm.1 - range * 0.8);
        let middle_slice: Vec<usize> = Vec::from(nifti.shape()).iter().map(|x| x / 2).collect();

        println!(" done!");
        println!("Image dimensions {:?}", nifti.shape());

        Self {
            running: true,
            file_path: file_path.to_string(),
            image_cache: brain::ImageCache::new(nifti),
            intensity_range: mm,
            slice_position: middle_slice,
            increment: 10,
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
                if self.slice_position[axis] + self.increment < self.image_cache.data.shape()[axis] {
                    self.slice_position[axis] += self.increment;
                }
            },
            _ => {}	
        }
    }

    pub fn decrement_slice(&mut self, axis: usize) {
        match axis {
            0..=2 => {
                if self.slice_position[axis] >= self.increment {
                    self.slice_position[axis] -= self.increment;
                }
            },
            _ => {}
        }
    }
}
