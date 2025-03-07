use anyhow::Result;
use image::{DynamicImage, RgbaImage};
use psd::Psd;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

pub(crate) fn convert_psd_to_dynamic_image(
    psd_path: &PathBuf,
    custom_width_enabled: bool,
    custom_width: u32,
) -> Result<DynamicImage> {
    let psd_data = fs::read(psd_path)?;
    let psd_file = Psd::from_bytes(&psd_data)?;
    let width = psd_file.width() as u32;
    let height = psd_file.height() as u32;
    
    // Process RGBA data in parallel with optimal chunk size
    let chunk_size = (width * 4).max(1).min(8192);
    let rgba_data: Vec<u8> = psd_file.rgba()
        .par_chunks(chunk_size as usize)
        .flat_map(|chunk| chunk.to_vec())
        .collect();

    let mut image = DynamicImage::ImageRgba8(
        RgbaImage::from_raw(width, height, rgba_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to create RGBA image from PSD data"))?,
    );

    if custom_width_enabled && custom_width > 0 && image.width() != custom_width {
        let aspect_ratio = image.width() as f32 / image.height() as f32;
        let new_height = (custom_width as f32 / aspect_ratio) as u32;
        image = image.resize_exact(
            custom_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
    }

    Ok(image)
}
