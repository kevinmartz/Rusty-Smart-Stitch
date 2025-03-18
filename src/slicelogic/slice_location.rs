use image::{DynamicImage, GrayImage};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct SliceLocation {
    pub scan_step: i32,
    pub edges: i32,
    pub sensitivity: i32,
    threshold_cache: RwLock<HashMap<i32, u8>>,
}

impl Default for SliceLocation {
    fn default() -> Self {
        Self {
            scan_step: 5,
            edges: 5,
            sensitivity: 100,
            threshold_cache: RwLock::new(HashMap::new()),
        }
    }
}

impl SliceLocation {
    pub fn new(scan_step: i32, edges: i32, sensitivity: i32) -> Self {
        Self {
            scan_step,
            edges,
            sensitivity,
            threshold_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn run(&self, combined_img: &DynamicImage, split_height: u32) -> Vec<u32> {
        let gray_img = combined_img.to_luma8();
        let threshold = {
            let mut cache = self.threshold_cache.write().unwrap();
            *cache.entry(self.sensitivity).or_insert_with(|| {
                ((255.0 * (1.0 - (self.sensitivity as f32 / 100.0))) as u32) as u8
            })
        };
        let last_row = gray_img.height();

        self.find_slice_locations(&gray_img, split_height, last_row, threshold)
    }

    fn find_slice_locations(
        &self,
        gray_img: &GrayImage,
        target_height: u32,
        last_row: u32,
        threshold: u8,
    ) -> Vec<u32> {
        let mut slice_locations = vec![0];
        let mut row = target_height;
        let mut move_up = true;

        while row < last_row {
            let can_slice = if row - slice_locations[slice_locations.len() - 1] > target_height / 2
            {
                self.check_row_for_slice(gray_img, row, threshold)
            } else {
                false
            };

            if can_slice {
                let slice_height = row - slice_locations[slice_locations.len() - 1];
                if slice_height > (target_height as f32 * 1.5) as u32 {
                    if let Some(better_row) = self.find_better_slice_point(
                        gray_img,
                        slice_locations[slice_locations.len() - 1],
                        row,
                        target_height,
                        threshold,
                    ) {
                        row = better_row;
                        slice_locations.push(row);
                    } else {
                        let forced_row = slice_locations[slice_locations.len() - 1]
                            + (target_height as f32 * 1.3) as u32;
                        if forced_row < last_row {
                            slice_locations.push(forced_row);
                        }
                        row = forced_row + target_height;
                    }
                } else {
                    slice_locations.push(row);
                    row += target_height;
                    move_up = true;
                    continue;
                }
            }

            if (row - slice_locations[slice_locations.len() - 1])
                <= (0.3 * target_height as f32) as u32
            {
                row = slice_locations[slice_locations.len() - 1] + target_height;
                move_up = false;
            }

            if move_up {
                row = row.saturating_sub(self.scan_step as u32);
            } else {
                row += self.scan_step as u32;
            }
        }

        self.handle_last_slice(&mut slice_locations, last_row, target_height);
        slice_locations
    }

    fn check_row_for_slice(&self, gray_img: &GrayImage, row: u32, threshold: u8) -> bool {
        let width = gray_img.width();
        let ignorable = self.edges as u32;
        let start_x = ignorable + 1;
        let end_x = width - ignorable;
        
        // Calculate optimal chunk size based on image width
        let chunk_size = (end_x - start_x).clamp(1, 512) as usize;
        
        (start_x..end_x)
            .into_par_iter()
            .chunks(chunk_size)
            .all(|chunk| {
                chunk.iter().all(|&x| {
                    let prev_pixel = gray_img.get_pixel(x, row).0[0];
                    let next_pixel = gray_img.get_pixel(x + 1, row).0[0];
                    (prev_pixel <= 9 && next_pixel <= 9) || next_pixel.abs_diff(prev_pixel) <= threshold
                })
            })
    }

    fn find_better_slice_point(
        &self,
        gray_img: &GrayImage,
        start_row: u32,
        end_row: u32,
        target_height: u32,
        threshold: u8,
    ) -> Option<u32> {
        let search_start = start_row + (target_height as f32 * 0.8) as u32;
        let search_end = (start_row + (target_height as f32 * 1.2) as u32).min(end_row);

        (search_start..search_end)
            .into_par_iter()
            .step_by(self.scan_step as usize)
            .find_map_first(|row| {
                if self.check_row_for_slice(gray_img, row, threshold) {
                    let height_diff =
                        ((row as i32) - (start_row as i32 + target_height as i32)).abs();
                    Some((height_diff, row))
                } else {
                    None
                }
            })
            .map(|(_, row)| row)
    }

    fn handle_last_slice(&self, slice_locations: &mut Vec<u32>, last_row: u32, target_height: u32) {
        if slice_locations[slice_locations.len() - 1] != last_row - 1 {
            let remaining_height = last_row - slice_locations[slice_locations.len() - 1];

            if remaining_height <= (target_height as f32 * 1.2) as u32 {
                slice_locations.push(last_row - 1);
            } else {
                let num_splits = (remaining_height as f32 / target_height as f32).ceil() as u32;
                let split_size = remaining_height / num_splits;

                let mut new_slices = Vec::with_capacity(num_splits as usize + 1);

                for i in 1..num_splits {
                    new_slices.push(slice_locations[slice_locations.len() - 1] + split_size * i);
                }
                new_slices.push(last_row - 1);

                slice_locations.extend(new_slices);
            }
        }
    }
}
