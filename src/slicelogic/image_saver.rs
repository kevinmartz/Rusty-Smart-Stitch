use anyhow::Result;
use image::{DynamicImage, EncodableLayout, ImageBuffer};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub(crate) fn save_slice(
    slice_img: &DynamicImage,
    slice_path: &PathBuf,
    output_format: &str,
    output_quality: u32,
) -> Result<()> {
    match output_format {
        "jpg" => save_as_jpg(slice_img, slice_path, output_quality),
        "png" => save_as_png(slice_img, slice_path),
        "webp" => save_as_webp(slice_img, slice_path, output_quality),
        "bmp" => save_as_bmp(slice_img, slice_path),
        _ => save_as_jpg(slice_img, slice_path, output_quality),
    }
}

fn save_as_jpg(slice_img: &DynamicImage, slice_path: &PathBuf, quality: u32) -> Result<()> {
    let rgba = slice_img.to_rgba8();
    let mut rgb_img = ImageBuffer::new(slice_img.width(), slice_img.height());

    for (x, y, pixel) in rgb_img.enumerate_pixels_mut() {
        let rgba_pixel = rgba.get_pixel(x, y);
        if rgba_pixel[3] < 128 {
            *pixel = image::Rgb([255, 255, 255]);
        } else {
            *pixel = image::Rgb([rgba_pixel[0], rgba_pixel[1], rgba_pixel[2]]);
        }
    }

    let output_file = std::fs::File::create(slice_path)?;
    let buf_writer = BufWriter::new(output_file);
    let mut encoder = if quality >= 100 {
        image::codecs::jpeg::JpegEncoder::new_with_quality(buf_writer, 100)
    } else {
        image::codecs::jpeg::JpegEncoder::new_with_quality(buf_writer, quality.clamp(1, 99) as u8)
    };
    encoder.encode(
        rgb_img.as_bytes(),
        rgb_img.width(),
        rgb_img.height(),
        image::ColorType::Rgb8.into(),
    )?;
    Ok(())
}

fn save_as_png(slice_img: &DynamicImage, slice_path: &PathBuf) -> Result<()> {
    let output_file = std::fs::File::create(slice_path)?;
    let buf_writer = BufWriter::new(output_file);
    slice_img.write_to(&mut BufWriter::new(buf_writer), image::ImageFormat::Png)?;
    Ok(())
}

fn save_as_webp(slice_img: &DynamicImage, slice_path: &PathBuf, quality: u32) -> Result<()> {
    const WEBP_MAX_DIMENSION: u32 = 16383;

    if slice_img.width() > WEBP_MAX_DIMENSION || slice_img.height() > WEBP_MAX_DIMENSION {
        return Err(anyhow::anyhow!(
            "Image dimensions ({}, {}) exceed WebP maximum limit of {} pixels. Try using a different output format or reduce height size.",
            slice_img.width(),
            slice_img.height(),
            WEBP_MAX_DIMENSION
        ));
    }

    let output_file = std::fs::File::create(slice_path)?;
    let mut buf_writer = BufWriter::new(output_file);
    let rgba = slice_img.to_rgba8();

    if quality >= 100 {
        let encoder = webp::Encoder::new(
            rgba.as_bytes(),
            webp::PixelLayout::Rgba,
            rgba.width(),
            rgba.height(),
        );
        let encoded = encoder.encode_lossless();
        buf_writer.write_all(encoded.as_bytes())?;
    } else {
        let quality = quality.clamp(1, 99) as f32;
        let encoder = webp::Encoder::new(
            rgba.as_bytes(),
            webp::PixelLayout::Rgba,
            rgba.width(),
            rgba.height(),
        );
        let encoded = encoder.encode(quality);
        buf_writer.write_all(encoded.as_bytes())?;
    }
    Ok(())
}

fn save_as_bmp(slice_img: &DynamicImage, slice_path: &PathBuf) -> Result<()> {
    let output_file = std::fs::File::create(slice_path)?;
    let buf_writer = BufWriter::new(output_file);
    slice_img.write_to(&mut BufWriter::new(buf_writer), image::ImageFormat::Bmp)?;
    Ok(())
}
