use super::{
    image_merger::merge_images_from_memory, image_saver::save_slice,
    psd_sup::convert_psd_to_dynamic_image,
};
use crate::slicelogic::SliceLocation;
use anyhow::Result;
use either::Either;
use image::DynamicImage;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct RustySmartStitch {
    pub input_paths: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub rough_output_height: u32,
    pub sensitivity: i32,
    pub scan_step: i32,
    pub edges: i32,
    pub output_format: String,
    pub output_quality: u32,
    pub custom_width_enabled: bool,
    pub custom_width: u32,
    pub upscale_enabled: bool,
    pub upscale_factor: u32,
    pub resize_enabled: bool,
    pub resize_width: u32,
    pub resize_height: u32,
}

impl RustySmartStitch {
    pub fn new(
        input_paths: Vec<PathBuf>,
        output_dir: PathBuf,
        rough_output_height: u32,
        sensitivity: i32,
        scan_step: i32,
        edges: i32,
        output_format: String,
        output_quality: u32,
        custom_width_enabled: bool,
        custom_width: u32,
        upscale_enabled: bool,
        upscale_factor: u32,
        resize_enabled: bool,
        resize_width: u32,
        resize_height: u32,
    ) -> Self {
        Self {
            input_paths,
            output_dir,
            rough_output_height,
            sensitivity,
            scan_step,
            edges,
            output_format,
            output_quality,
            custom_width_enabled,
            custom_width,
            upscale_enabled,
            upscale_factor,
            resize_enabled,
            resize_width,
            resize_height,
        }
    }

    fn convert_psd_to_dynamic_image(&self, psd_path: &PathBuf) -> Result<DynamicImage> {
        convert_psd_to_dynamic_image(psd_path, self.custom_width_enabled, self.custom_width)
    }

    pub fn process<F>(&self, progress_callback: Option<F>) -> Result<()>
    where
        F: Fn(f32) + Send + Sync,
    {
        let report_progress = |progress: f32| {
            if let Some(callback) = &progress_callback {
                callback(progress);
            }
        };

        let mut image_data = Vec::new();

        // File loading phase (0-30%)
        let total_files = self.input_paths.len();
        for (i, path) in self.input_paths.iter().enumerate() {
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "psd" {
                    match self.convert_psd_to_dynamic_image(path) {
                        Ok(image) => image_data.push(Either::Left(image)),
                        Err(e) => return Err(anyhow::anyhow!("Failed to process PSD file: {}", e)),
                    }
                } else {
                    image_data.push(Either::Right(path.clone()));
                }
            } else {
                image_data.push(Either::Right(path.clone()));
            }
            report_progress(0.3 * (i + 1) as f32 / total_files as f32);
        }

        // Image merging (30-40%)
        report_progress(0.3);
        let merged_image = if image_data.len() > 1 {
            let result = self.merge_images_from_memory(&image_data)?;
            report_progress(0.4);
            std::mem::drop(image_data);
            result
        } else {
            let result = match &image_data[0] {
                Either::Left(image) => image.clone(),
                Either::Right(path) => image::open(path)?,
            };
            report_progress(0.4);
            result
        };

        // Slice detection (40-50%)
        let detector = SliceLocation::new(self.scan_step, self.edges, self.sensitivity);
        let slice_locations = detector.run(&merged_image, self.rough_output_height);
        report_progress(0.5);

        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Slice processing (50-100%)
        let total_slices = slice_locations.len() - 1;
        let processed_slices = Arc::new(AtomicUsize::new(0));
        let progress_callback = Arc::new(report_progress);

        // Process slices in parallel
        slice_locations
            .par_windows(2)
            .enumerate()
            .try_for_each(|(i, window)| {
                let progress_callback = Arc::clone(&progress_callback);
                let processed_slices = Arc::clone(&processed_slices);

                let result: Result<()> = (|| {
                    let (start_y, end_y) = (window[0], window[1]);
                    let mut slice_img =
                        merged_image.crop_imm(0, start_y, merged_image.width(), end_y - start_y);

                    if slice_img.color() == image::ColorType::Rgba8
                        && !matches!(self.output_format.as_str(), "webp" | "png")
                    {
                        slice_img = DynamicImage::ImageRgb8(slice_img.to_rgb8());
                    }

                    slice_img = self.process_image(&slice_img)?;

                    let slice_name = format!("{:03}.{}", i + 1, self.output_format);
                    let slice_path = self.output_dir.join(slice_name);

                    self.save_slice(&slice_img, &slice_path)?;

                    let completed = processed_slices.fetch_add(1, Ordering::SeqCst) + 1;
                    let slice_progress = 0.5 + (0.5 * completed as f32 / total_slices as f32);
                    progress_callback(slice_progress);

                    Ok(())
                })();

                result
            })?;

        report_progress(1.0);
        Ok(())
    }

    fn merge_images_from_memory(
        &self,
        image_data: &[Either<DynamicImage, PathBuf>],
    ) -> Result<DynamicImage> {
        merge_images_from_memory(
            image_data,
            self.custom_width_enabled,
            self.custom_width,
            &self.output_format,
        )
    }

    fn save_slice(&self, slice_img: &DynamicImage, slice_path: &PathBuf) -> Result<()> {
        save_slice(
            slice_img,
            slice_path,
            &self.output_format,
            self.output_quality,
        )
    }

    fn process_image(&self, image: &DynamicImage) -> Result<DynamicImage> {
        let mut processed = image.clone();

        if self.custom_width_enabled
            && self.custom_width > 0
            && processed.width() != self.custom_width
        {
            let aspect_ratio = processed.width() as f32 / processed.height() as f32;
            let new_height = (self.custom_width as f32 / aspect_ratio) as u32;
            processed = processed.resize_exact(
                self.custom_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            );
        }

        if self.upscale_enabled && self.upscale_factor > 1 {
            let new_width = processed.width() * self.upscale_factor;
            let new_height = processed.height() * self.upscale_factor;
            processed =
                processed.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
        }

        if self.resize_enabled && self.resize_width > 0 && self.resize_height > 0 {
            processed = processed.resize(
                self.resize_width,
                self.resize_height,
                image::imageops::FilterType::Lanczos3,
            );
        }

        Ok(processed)
    }
}
