use crate::slicelogic::CHUNK_SIZE;
use anyhow::Result;
use either::Either;
use image::{DynamicImage, ImageBuffer};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub(crate) fn merge_images_from_memory(
    image_data: &[Either<DynamicImage, PathBuf>],
    custom_width_enabled: bool,
    custom_width: u32,
    output_format: &str,
) -> Result<DynamicImage> {
    let images_buffer = Arc::new(Mutex::new(Vec::with_capacity(image_data.len())));

    image_data
        .par_chunks(CHUNK_SIZE.min(image_data.len()))
        .try_for_each(|chunk| -> Result<()> {
            let local_images: Result<Vec<_>> = chunk
                .par_iter()
                .map(|item| -> Result<DynamicImage> {
                    let mut img = match item {
                        Either::Left(image) => image.clone(),
                        Either::Right(path) => image::open(path)?,
                    };

                    if custom_width_enabled && custom_width > 0 && img.width() != custom_width {
                        let aspect_ratio = img.width() as f32 / img.height() as f32;
                        let new_height = (custom_width as f32 / aspect_ratio) as u32;
                        img = img.resize_exact(
                            custom_width,
                            new_height,
                            image::imageops::FilterType::Lanczos3,
                        );
                    }

                    Ok(img)
                })
                .collect();

            images_buffer.lock().unwrap().extend(local_images?);
            Ok(())
        })?;

    let images = images_buffer.lock().unwrap();

    let total_height: u32 = images.iter().map(|img| img.height()).sum();
    let max_width: u32 = images.iter().map(|img| img.width()).max().unwrap_or(0);

    match output_format {
        "png" | "webp" => merge_images_rgba(&images, max_width, total_height),
        _ => merge_images_rgb(&images, max_width, total_height),
    }
}

pub(crate) fn merge_images_rgba(
    images: &[DynamicImage],
    width: u32,
    height: u32,
) -> Result<DynamicImage> {
    let merged = Arc::new(Mutex::new(ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(
        width, height,
    )));
    
    // Initialize with transparent background
    {
        let mut buffer = merged.lock().unwrap();
        buffer.par_chunks_mut(4).for_each(|p| {
            p.copy_from_slice(&[255, 255, 255, 0]);
        });
    }

    let mut current_offset = 0;
    
    for img in images {
        let img_height = img.height();
        let rgba = img.to_rgba8();
        
        // Process each row of the current image
        (0..img_height).into_par_iter().for_each(|y| {
            let mut buffer = merged.lock().unwrap();
            for x in 0..img.width() {
                let pixel = rgba.get_pixel(x, y);
                buffer.put_pixel(x, y + current_offset, *pixel);
            }
        });
        
        current_offset += img_height;
    }

    Ok(DynamicImage::ImageRgba8(
        Arc::try_unwrap(merged).unwrap().into_inner().unwrap(),
    ))
}

pub(crate) fn merge_images_rgb(
    images: &[DynamicImage],
    width: u32,
    height: u32,
) -> Result<DynamicImage> {
    let merged = Arc::new(Mutex::new(ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(
        width, height,
    )));

    {
        let mut merged = merged.lock().unwrap();
        merged.par_chunks_mut(3).for_each(|p| {
            p.copy_from_slice(&[255, 255, 255]);
        });
    }

    let y_offset = Arc::new(AtomicUsize::new(0));

    for img in images {
        let rgba = img.to_rgba8();
        let merged_clone = Arc::clone(&merged);
        let current_offset = y_offset.load(Ordering::SeqCst);

        let chunk_height = 16.min(img.height()); // Process 16 scanlines at a time eh 16 is fine
        let num_chunks = (img.height() + chunk_height - 1) / chunk_height;

        (0..num_chunks).into_par_iter().for_each(move |chunk_idx| {
            let start_y = chunk_idx * chunk_height;
            let end_y = (start_y + chunk_height).min(img.height());
            let mut merged = merged_clone.lock().unwrap();

            for y in start_y..end_y {
                for x in 0..img.width() {
                    let rgba_pixel = rgba.get_pixel(x, y);
                    let alpha = rgba_pixel[3] as f32 / 255.0;

                    if alpha > 0.0 {
                        let bg_pixel = merged.get_pixel(x, y + current_offset as u32);
                        let new_pixel = image::Rgb([
                            ((rgba_pixel[0] as f32 * alpha + bg_pixel[0] as f32 * (1.0 - alpha))
                                as u8),
                            ((rgba_pixel[1] as f32 * alpha + bg_pixel[1] as f32 * (1.0 - alpha))
                                as u8),
                            ((rgba_pixel[2] as f32 * alpha + bg_pixel[2] as f32 * (1.0 - alpha))
                                as u8),
                        ]);
                        merged.put_pixel(x, y + current_offset as u32, new_pixel);
                    }
                }
            }
        });

        y_offset.fetch_add(img.height() as usize, Ordering::SeqCst);
    }

    Ok(DynamicImage::ImageRgb8(
        Arc::try_unwrap(merged).unwrap().into_inner().unwrap(),
    ))
}
