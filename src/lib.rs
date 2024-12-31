use anyhow::Result;
use image::{DynamicImage, GrayImage, ImageBuffer};
use image::EncodableLayout;
use psd::Psd;
use rayon::prelude::*;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;
use std::io::Cursor;
use std::io::Write;
use either::Either;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::cell::RefCell;

pub mod waifu2x;

const CHUNK_SIZE: usize = 4096;
const PARALLEL_THRESHOLD: usize = 1000;

pub struct SliceLocation {
    pub scan_step: i32,
    pub edges: i32,
    pub sensitivity: i32,
    threshold_cache: RefCell<HashMap<i32, u8>>,
}

impl Default for SliceLocation {
    fn default() -> Self {
        Self {
            scan_step: 5,
            edges: 5,
            sensitivity: 100,
            threshold_cache: RefCell::new(HashMap::new()),
        }
    }
}

impl SliceLocation {
    pub fn new(scan_step: i32, edges: i32, sensitivity: i32) -> Self {
        Self {
            scan_step,
            edges,
            sensitivity,
            threshold_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn run(&self, combined_img: &DynamicImage, split_height: u32) -> Vec<u32> {
        let gray_img = combined_img.to_luma8();
        let threshold = {
            let mut cache = self.threshold_cache.borrow_mut();
            if let Some(&t) = cache.get(&self.sensitivity) {
                t
            } else {
                let t = ((255.0 * (1.0 - (self.sensitivity as f32 / 100.0))) as u32) as u8;
                cache.insert(self.sensitivity, t);
                t
            }
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
        let mut slice_locations = vec![0];  // Start from the top
        let mut row = target_height;        // First guess is at target height
        let mut move_up = true;             // Whether to look up or down for better spots
    
        while row < last_row {
            let can_slice = self.check_row_for_slice(gray_img, row, threshold);
    
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
                        // Found a shit slice point, fuck it - just force it at 130% height
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

            // If we're too close to the last slice, just skip ahead
            // No point looking for slices in tiny gaps
            if (row - slice_locations[slice_locations.len() - 1]) <= (0.3 * target_height as f32) as u32 {
                row = slice_locations[slice_locations.len() - 1] + target_height;
                move_up = false;
            }
            if move_up {
                row = row.saturating_sub(self.scan_step as u32);
                continue;
            }
            row += self.scan_step as u32;
        }
    
        // Handle whatever's left at the bottom
        self.handle_last_slice(&mut slice_locations, last_row, target_height);
        slice_locations
    }

    // Looking for either:
    // - black pixels (probably panel borders)
    // - consistent color (probably whitespace/gaps)
    fn check_row_for_slice(&self, gray_img: &GrayImage, row: u32, threshold: u8) -> bool {
        let width = gray_img.width() as i32;
        let ignorable = self.edges;
        
        // Skip some pixels at the edges cuz they're usually messy
        let mut x = (ignorable + 1) as u32;
        while x < (width - ignorable) as u32 {
            let prev_pixel = gray_img.get_pixel(x - 1, row)[0];
            let next_pixel = gray_img.get_pixel(x, row)[0];
            
            // Either both pixels are black-ish (<=9) or they're similar enough
            if !((prev_pixel <= 9 && next_pixel <= 9) || 
                next_pixel.abs_diff(prev_pixel) <= threshold) {
                return false;
            }
            x += 1;
        }
        true
    }

    // When the normal slice point sucks, try to find a better one nearby
    // Looks between 80% and 120% of the target height
    fn find_better_slice_point(
        &self,
        gray_img: &GrayImage,
        start_row: u32,
        end_row: u32,
        target_height: u32,
        threshold: u8,
    ) -> Option<u32> {
        // Search range: 80% to 120% of target height
        let search_start = start_row + (target_height as f32 * 0.8) as u32;
        let search_end = (start_row + (target_height as f32 * 1.2) as u32).min(end_row);

        let mut best_row = None;
        let mut min_diff = f32::INFINITY;

        for row in (search_start..search_end).step_by(self.scan_step as usize) {
            if self.check_row_for_slice(gray_img, row, threshold) {
                let height_diff =
                    ((row as i32) - (start_row as i32 + target_height as i32)).abs() as f32;
                if height_diff < min_diff {
                    min_diff = height_diff;
                    best_row = Some(row);
                }
            }
        }
        best_row
    }

    // Deals with whatever's left at the bottom of the image
    fn handle_last_slice(&self, slice_locations: &mut Vec<u32>, last_row: u32, target_height: u32) {
        if slice_locations[slice_locations.len() - 1] != last_row - 1 {
            if last_row - slice_locations[slice_locations.len() - 1]
                <= (target_height as f32 * 1.2) as u32
            {
                slice_locations.push(last_row - 1);
            } else {
                let remaining_height = last_row - slice_locations[slice_locations.len() - 1];
                let num_splits = (remaining_height as f32 / target_height as f32).ceil() as u32;
                let split_size = remaining_height / num_splits;

                for _i in 1..num_splits {
                    slice_locations.push(slice_locations[slice_locations.len() - 1] + split_size);
                }
                slice_locations.append(&mut vec![last_row - 1]);
            }
        }
    }
}

// Main struct that handles all the image processing
// Yeah... lots of fields but each one does something important
pub struct RustySmartStitch {
    pub input_paths: Vec<PathBuf>,        // Where to get the images from
    pub output_dir: PathBuf,              // Where to dump the results
    pub rough_output_height: u32,         // target height for slices
    pub sensitivity: i32,                 // How picky to be about finding slice points
    pub scan_step: i32,                   // Same shit as in SliceLocation
    pub edges: i32,                       // Edge pixels to ignore when detecting slices
    pub output_format: String,            // jpg, png, etc
    pub output_quality: u32,              // Higher = better quality but bigger files
    pub custom_width_enabled: bool,       // force a specific width
    pub custom_width: u32,                // The width to force if enabled
    pub upscale_enabled: bool,            // Make images bigger
    pub upscale_factor: u32,              // How much bigger
    pub resize_enabled: bool,             // Resize to specific dimensions
    pub resize_width: u32,                // Target width for resize
    pub resize_height: u32,               // Target height for resize
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

    // Convert PSD to PNG (why png? well its lossless)
    fn convert_psd_to_png(&self, psd_path: &PathBuf) -> Result<Vec<u8>> {
        let psd_data = fs::read(psd_path)?;
        let psd_file = Psd::from_bytes(&psd_data)?;

        let width = psd_file.width() as u32;
        let height = psd_file.height() as u32;
        let rgba = psd_file.rgba();

        // Process chunks in parallel
        let chunk_size = (width * 4) as usize;
        let processed_data: Vec<u8> = rgba
            .par_chunks(chunk_size)
            .flat_map(|chunk| chunk.to_vec())
            .collect();

        let img = ImageBuffer::from_raw(width, height, processed_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from PSD"))?;

        let mut dynamic_image = DynamicImage::ImageRgba8(img);

        if self.custom_width_enabled && self.custom_width > 0 && dynamic_image.width() != self.custom_width {
            let aspect_ratio = dynamic_image.width() as f32 / dynamic_image.height() as f32;
            let new_height = (self.custom_width as f32 / aspect_ratio) as u32;
            dynamic_image = dynamic_image.resize_exact(
                self.custom_width,
                new_height,
                image::imageops::FilterType::Lanczos3
            );
        }

        // Convert to PNG and return as bytes
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        dynamic_image.write_to(&mut cursor, image::ImageFormat::Png)?;

        Ok(buffer)
    }

    pub fn process(&self) -> Result<()> {
        let mut png_data = Vec::new();

        // convert any PSDs to PNGs in memory
        for path in &self.input_paths {
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "psd" {
                    match self.convert_psd_to_png(path) {
                        Ok(buffer) => {
                            png_data.push(Either::Left(buffer));
                        }
                        Err(e) => return Err(anyhow::anyhow!("Failed to convert PSD file: {}", e)),
                    };
                } else {
                    png_data.push(Either::Right(path.clone()));
                }
            } else {
                png_data.push(Either::Right(path.clone()));
            }
        }

        // more than one image, merges them
        // if single image, loads it
        let merged_image = if png_data.len() > 1 {
            self.merge_images_from_memory(&png_data)?
        } else {
            match &png_data[0] {
                Either::Left(buffer) => image::load_from_memory(buffer)?,
                Either::Right(path) => image::open(path)?,
            }
        };

        self.split_image(&merged_image)?;

        Ok(())
    }

    // Merge images in memory
    fn merge_images_from_memory(&self, png_data: &[Either<Vec<u8>, PathBuf>]) -> Result<DynamicImage> {
        let images_buffer = Arc::new(Mutex::new(Vec::with_capacity(png_data.len())));
        
        // Process chunks of images in parallel
        png_data.par_chunks(CHUNK_SIZE.min(png_data.len()))
            .try_for_each(|chunk| -> Result<()> {
                let mut local_images = Vec::with_capacity(chunk.len());
                
                for item in chunk {
                    let mut img = match item {
                        Either::Left(buffer) => image::load_from_memory(buffer)?,
                        Either::Right(path) => image::open(path)?,
                    };

                    // resize if needed
                    if self.custom_width_enabled && self.custom_width > 0 && img.width() != self.custom_width {
                        let aspect_ratio = img.width() as f32 / img.height() as f32;
                        let new_height = (self.custom_width as f32 / aspect_ratio) as u32;
                        img = img.resize_exact(
                            self.custom_width,
                            new_height,
                            image::imageops::FilterType::Lanczos3
                        );
                    }
                    
                    local_images.push(img);
                }
                
                images_buffer.lock().unwrap().extend(local_images);
                Ok(())
            })?;

        let images = images_buffer.lock().unwrap();
        
        let total_height: u32 = images.iter().map(|img| img.height()).sum();
        let max_width: u32 = images.iter().map(|img| img.width()).max().unwrap_or(0);

        let result = if self.output_format == "png" || self.output_format == "webp" {
            self.merge_images_rgba(&images, max_width, total_height)?
        } else {
            self.merge_images_rgb(&images, max_width, total_height)?
        };

        Ok(result)
    }

    // Merge images with transparency - pretty straightforward
    fn merge_images_rgba(&self, images: &[DynamicImage], width: u32, height: u32) -> Result<DynamicImage> {
        let mut merged = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(width, height);
        
        merged.pixels_mut().for_each(|p| *p = image::Rgba([255, 255, 255, 0]));

        let mut y_offset = 0;
        for img in images {
            let rgba = img.to_rgba8();
            image::imageops::replace(&mut merged, &rgba, 0, y_offset as i64);
            y_offset += img.height();
        }

        Ok(DynamicImage::ImageRgba8(merged))
    }

    // Merge images without transparency
    fn merge_images_rgb(&self, images: &[DynamicImage], width: u32, height: u32) -> Result<DynamicImage> {
        let merged = Arc::new(Mutex::new(ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(width, height)));
        
        {
            let mut merged = merged.lock().unwrap();
            merged.pixels_mut().for_each(|p| *p = image::Rgb([255, 255, 255]));
        }

        let y_offset = Arc::new(AtomicUsize::new(0));
        
        for img in images {
            let rgba = img.to_rgba8();
            let merged_clone = Arc::clone(&merged);
            let current_offset = y_offset.load(Ordering::SeqCst);
            
            // If image is big enough, process scanlines in parallel
            // Kinda overkill for small images tho but eh
            if img.height() > PARALLEL_THRESHOLD as u32 {
                (0..img.height()).into_par_iter().for_each(move |y| {
                    let mut merged = merged_clone.lock().unwrap();
                    for x in 0..img.width() {
                        let rgba_pixel = rgba.get_pixel(x, y);
                        let alpha = rgba_pixel[3] as f32 / 255.0;
                        
                        if alpha > 0.0 {
                            let bg_pixel = merged.get_pixel(x, y + current_offset as u32);
                            let new_pixel = image::Rgb([
                                ((rgba_pixel[0] as f32 * alpha + bg_pixel[0] as f32 * (1.0 - alpha)) as u8),
                                ((rgba_pixel[1] as f32 * alpha + bg_pixel[1] as f32 * (1.0 - alpha)) as u8),
                                ((rgba_pixel[2] as f32 * alpha + bg_pixel[2] as f32 * (1.0 - alpha)) as u8),
                            ]);
                            merged.put_pixel(x, y + current_offset as u32, new_pixel);
                        }
                    }
                });
            } else {
                let mut merged = merged.lock().unwrap();
                for y in 0..img.height() {
                    for x in 0..img.width() {
                        let rgba_pixel = rgba.get_pixel(x, y);
                        let alpha = rgba_pixel[3] as f32 / 255.0;
                        
                        if alpha > 0.0 {
                            let bg_pixel = merged.get_pixel(x, y + current_offset as u32);
                            let new_pixel = image::Rgb([
                                ((rgba_pixel[0] as f32 * alpha + bg_pixel[0] as f32 * (1.0 - alpha)) as u8),
                                ((rgba_pixel[1] as f32 * alpha + bg_pixel[1] as f32 * (1.0 - alpha)) as u8),
                                ((rgba_pixel[2] as f32 * alpha + bg_pixel[2] as f32 * (1.0 - alpha)) as u8),
                            ]);
                            merged.put_pixel(x, y + current_offset as u32, new_pixel);
                        }
                    }
                }
            }
            y_offset.fetch_add(img.height() as usize, Ordering::SeqCst);
        }

        Ok(DynamicImage::ImageRgb8(Arc::try_unwrap(merged)
            .unwrap()
            .into_inner()
            .unwrap()))
    }

    fn split_image(&self, image: &DynamicImage) -> Result<()> {
        let detector = SliceLocation::new(self.scan_step, self.edges, self.sensitivity);
        let slice_locations = detector.run(image, self.rough_output_height);
        fs::create_dir_all(&self.output_dir)?;

        // Process slices in parallel
        slice_locations
            .par_windows(2)
            .enumerate()
            .try_for_each(|(i, window)| -> Result<()> {
                let (start_y, end_y) = (window[0], window[1]);
                let mut slice_img = image.crop_imm(0, start_y, image.width(), end_y - start_y);

                // Only convert to RGB for formats that don't support transparency
                if slice_img.color() == image::ColorType::Rgba8 && 
                   !matches!(self.output_format.as_str(), "webp" | "png") {
                    slice_img = DynamicImage::ImageRgb8(slice_img.to_rgb8());
                }

                // Apply processing if any settings are enabled
                slice_img = self.process_image(&slice_img)?;

                let slice_name = format!("{:03}.{}", i + 1, self.output_format);
                let slice_path = self.output_dir.join(slice_name);

                match self.output_format.as_str() {
                    "jpg" => {
                        // For JPG, handle transparency by removing alpha channel
                        let rgba = slice_img.to_rgba8();
                        let mut rgb_img = ImageBuffer::new(slice_img.width(), slice_img.height());
                        
                        for (x, y, pixel) in rgb_img.enumerate_pixels_mut() {
                            let rgba_pixel = rgba.get_pixel(x, y);
                            if rgba_pixel[3] < 128 {
                                // If mostly transparent, we will use white
                                *pixel = image::Rgb([255, 255, 255]);
                            } else {
                                // If mostly opaque, we will use the color values directly
                                *pixel = image::Rgb([
                                    rgba_pixel[0],
                                    rgba_pixel[1],
                                    rgba_pixel[2],
                                ]);
                            }
                        }
                        
                        let output_file = std::fs::File::create(&slice_path)?;
                        let buf_writer = BufWriter::new(output_file);
                        let mut encoder = if self.output_quality >= 100 {
                            // maximum quality settings for lossless
                            image::codecs::jpeg::JpegEncoder::new_with_quality(
                                buf_writer,
                                100
                            )
                        } else {
                            image::codecs::jpeg::JpegEncoder::new_with_quality(
                                buf_writer,
                                self.output_quality.clamp(1, 99) as u8,
                            )
                        };
                        encoder.encode(
                            rgb_img.as_bytes(),
                            rgb_img.width(),
                            rgb_img.height(),
                            image::ColorType::Rgb8.into(),
                        )?;
                    }
                    "png" => {
                        // For PNG, preserve transparency
                        let output_file = std::fs::File::create(&slice_path)?;
                        let buf_writer = BufWriter::new(output_file);
                        slice_img
                            .write_to(&mut BufWriter::new(buf_writer), image::ImageFormat::Png)?;
                    }
                    "webp" => {
                        // WebP supports transparency and quality
                        let output_file = std::fs::File::create(&slice_path)?;
                        let mut buf_writer = BufWriter::new(output_file);
                        let rgba = slice_img.to_rgba8();
                        
                        if self.output_quality >= 100 {
                            // lossless mode for quality 100
                            let encoder = webp::Encoder::new(
                                rgba.as_bytes(),
                                webp::PixelLayout::Rgba,
                                rgba.width(),
                                rgba.height()
                            );
                            let encoded = encoder.encode_lossless();
                            buf_writer.write_all(encoded.as_bytes())?;
                        } else {
                            let quality = self.output_quality.clamp(1, 99) as f32;
                            let encoder = webp::Encoder::new(
                                rgba.as_bytes(),
                                webp::PixelLayout::Rgba,
                                rgba.width(),
                                rgba.height()
                            );
                            let encoded = encoder.encode(quality);
                            buf_writer.write_all(encoded.as_bytes())?;
                        }
                    }
                    "bmp" => {
                        let rgb_img = slice_img.to_rgb8();
                        let output_file = std::fs::File::create(&slice_path)?;
                        let buf_writer = BufWriter::new(output_file);
                        rgb_img
                            .write_to(&mut BufWriter::new(buf_writer), image::ImageFormat::Bmp)?;
                    }
                    _ => {
                        let rgba = slice_img.to_rgba8();
                        let mut rgb_img = ImageBuffer::new(slice_img.width(), slice_img.height());
                        
                        for (x, y, pixel) in rgb_img.enumerate_pixels_mut() {
                            let rgba_pixel = rgba.get_pixel(x, y);
                            if rgba_pixel[3] < 128 {
                                // If mostly transparent, we will use white
                                *pixel = image::Rgb([255, 255, 255]);
                            } else {
                                // If mostly opaque, we will use the color values directly
                                *pixel = image::Rgb([
                                    rgba_pixel[0],
                                    rgba_pixel[1],
                                    rgba_pixel[2],
                                ]);
                            }
                        }
                        
                        let output_file = std::fs::File::create(&slice_path)?;
                        let buf_writer = BufWriter::new(output_file);
                        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                            buf_writer,
                            self.output_quality.clamp(1, 100) as u8,
                        );
                        encoder.encode(
                            rgb_img.as_bytes(),
                            rgb_img.width(),
                            rgb_img.height(),
                            image::ColorType::Rgb8.into(),
                        )?;
                    }
                }
                Ok(())
            })?;

        Ok(())
    }

    pub fn process_with_progress<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(f32) + Send + Sync,
    {
        let mut png_data = Vec::new();

        // File loading phase (0-30%)
        let total_files = self.input_paths.len();
        for (i, path) in self.input_paths.iter().enumerate() {
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "psd" {
                    match self.convert_psd_to_png(path) {
                        Ok(buffer) => png_data.push(Either::Left(buffer)),
                        Err(e) => return Err(anyhow::anyhow!("Failed to convert PSD file: {}", e))
                    }
                } else {
                    png_data.push(Either::Right(path.clone()));
                }
            } else {
                png_data.push(Either::Right(path.clone()));
            }
            progress_callback(0.3 * (i + 1) as f32 / total_files as f32);
        }

        // Image merging (30-40%)
        progress_callback(0.3);
        let merged_image = if png_data.len() > 1 {
            let result = self.merge_images_from_memory(&png_data)?;
            progress_callback(0.4);
            result
        } else {
            let result = match &png_data[0] {
                Either::Left(buffer) => image::load_from_memory(buffer)?,
                Either::Right(path) => image::open(path)?,
            };
            progress_callback(0.4);
            result
        };

        // Slice detection (40-50%)
        let detector = SliceLocation::new(self.scan_step, self.edges, self.sensitivity);
        let slice_locations = detector.run(&merged_image, self.rough_output_height);
        progress_callback(0.5);

        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Slice processing (50-100%)
        let total_slices = slice_locations.len() - 1;
        let processed_slices = Arc::new(AtomicUsize::new(0));
        let progress_callback = Arc::new(progress_callback);

        // Process slices in parallel
        slice_locations
            .par_windows(2)
            .enumerate()
            .try_for_each(|(i, window)| {
                let progress_callback = Arc::clone(&progress_callback);
                let processed_slices = Arc::clone(&processed_slices);
                
                let result: Result<()> = (|| {
                    let (start_y, end_y) = (window[0], window[1]);
                    let mut slice_img = merged_image.crop_imm(0, start_y, merged_image.width(), end_y - start_y);

                    // Only convert to RGB for formats that don't support transparency
                    if slice_img.color() == image::ColorType::Rgba8 && 
                       !matches!(self.output_format.as_str(), "webp" | "png") {
                        slice_img = DynamicImage::ImageRgb8(slice_img.to_rgb8());
                    }

                    // Apply processing if any settings are enabled
                    slice_img = self.process_image(&slice_img)?;

                    let slice_name = format!("{:03}.{}", i + 1, self.output_format);
                    let slice_path = self.output_dir.join(slice_name);

                    match self.output_format.as_str() {
                        "jpg" => {
                            // For JPG, handle transparency by removing alpha channel
                            let rgba = slice_img.to_rgba8();
                            let mut rgb_img = ImageBuffer::new(slice_img.width(), slice_img.height());
                            
                            for (x, y, pixel) in rgb_img.enumerate_pixels_mut() {
                                let rgba_pixel = rgba.get_pixel(x, y);
                                if rgba_pixel[3] < 128 {
                                    // If mostly transparent, we will use white
                                    *pixel = image::Rgb([255, 255, 255]);
                                } else {
                                    // If mostly opaque, we will use the color values directly
                                    *pixel = image::Rgb([
                                        rgba_pixel[0],
                                        rgba_pixel[1],
                                        rgba_pixel[2],
                                    ]);
                                }
                            }
                            
                            let output_file = std::fs::File::create(&slice_path)?;
                            let buf_writer = BufWriter::new(output_file);
                            let mut encoder = if self.output_quality >= 100 {
                                // maximum quality settings for lossless
                                image::codecs::jpeg::JpegEncoder::new_with_quality(
                                    buf_writer,
                                    100
                                )
                            } else {
                                image::codecs::jpeg::JpegEncoder::new_with_quality(
                                    buf_writer,
                                    self.output_quality.clamp(1, 99) as u8,
                                )
                            };
                            encoder.encode(
                                rgb_img.as_bytes(),
                                rgb_img.width(),
                                rgb_img.height(),
                                image::ColorType::Rgb8.into(),
                            )?;
                        }
                        "png" => {
                            let output_file = std::fs::File::create(&slice_path)?;
                            let buf_writer = BufWriter::new(output_file);
                            slice_img
                                .write_to(&mut BufWriter::new(buf_writer), image::ImageFormat::Png)?;
                        }
                        "webp" => {
                            let output_file = std::fs::File::create(&slice_path)?;
                            let mut buf_writer = BufWriter::new(output_file);
                            let rgba = slice_img.to_rgba8();
                            
                            if self.output_quality >= 100 {
                                // lossless mode for quality 100
                                let encoder = webp::Encoder::new(
                                    rgba.as_bytes(),
                                    webp::PixelLayout::Rgba,
                                    rgba.width(),
                                    rgba.height()
                                );
                                let encoded = encoder.encode_lossless();
                                buf_writer.write_all(encoded.as_bytes())?;
                            } else {
                                let quality = self.output_quality.clamp(1, 99) as f32;
                                let encoder = webp::Encoder::new(
                                    rgba.as_bytes(),
                                    webp::PixelLayout::Rgba,
                                    rgba.width(),
                                    rgba.height()
                                );
                                let encoded = encoder.encode(quality);
                                buf_writer.write_all(encoded.as_bytes())?;
                            }
                        }
                        "bmp" => {
                            let output_file = std::fs::File::create(&slice_path)?;
                            let buf_writer = BufWriter::new(output_file);
                            slice_img
                                .write_to(&mut BufWriter::new(buf_writer), image::ImageFormat::Bmp)?;
                        }
                        _ => {
                            let output_file = std::fs::File::create(&slice_path)?;
                            let buf_writer = BufWriter::new(output_file);
                            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                                buf_writer,
                                self.output_quality.clamp(1, 100) as u8,
                            );
                            encoder.encode(
                                slice_img.as_bytes(),
                                slice_img.width(),
                                slice_img.height(),
                                slice_img.color().into(),
                            )?;
                        }
                    }

                    // Update progress (50-100%)
                    let completed = processed_slices.fetch_add(1, Ordering::SeqCst) + 1;
                    let slice_progress = 0.5 + (0.5 * completed as f32 / total_slices as f32);
                    progress_callback(slice_progress);

                    Ok(())
                })();

                result
            })?;

        progress_callback(1.0);

        Ok(())
    }

    fn process_image(&self, image: &DynamicImage) -> Result<DynamicImage> {
        let mut processed = image.clone();

        if self.custom_width_enabled && self.custom_width > 0 && processed.width() != self.custom_width {
            let aspect_ratio = processed.width() as f32 / processed.height() as f32;
            let new_height = (self.custom_width as f32 / aspect_ratio) as u32;
            processed = processed.resize_exact(
                self.custom_width,
                new_height,
                image::imageops::FilterType::Lanczos3
            );
        }

        if self.upscale_enabled && self.upscale_factor > 1 {
            let new_width = processed.width() * self.upscale_factor;
            let new_height = processed.height() * self.upscale_factor;
            processed = processed.resize(
                new_width, 
                new_height, 
                image::imageops::FilterType::Lanczos3
            );
        }

        if self.resize_enabled && self.resize_width > 0 && self.resize_height > 0 {
            processed = processed.resize(
                self.resize_width, 
                self.resize_height, 
                image::imageops::FilterType::Lanczos3
            );
        }

        Ok(processed)
    }
}
