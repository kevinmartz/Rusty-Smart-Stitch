use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    // Main settings
    pub rough_output_height: String,
    pub sensitivity: String,
    pub scan_step: String,
    pub edges: String,
    pub output_format: String,
    pub output_quality: String,

    // Advanced settings
    pub custom_width_enabled: bool,
    pub custom_width: String,
    pub upscale_enabled: bool,
    pub upscale_factor: u32,
    pub resize_enabled: bool,
    pub resize_width: String,
    pub resize_height: String,

    // Waifu2x settings
    pub waifu2x_enabled: bool,
    pub waifu2x_mode: String,
    pub waifu2x_noise_level: String,
    pub waifu2x_scale_mode: String,
    pub waifu2x_scale_ratio: String,
    pub waifu2x_scale_width: String,
    pub waifu2x_scale_height: String,
    pub waifu2x_model: String,
    pub waifu2x_tta: bool,
    pub waifu2x_crop_size: String,
    pub waifu2x_batch_size: String,
    pub waifu2x_process: String,
    pub waifu2x_output_depth: String,
}

impl Profile {
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let profile = serde_json::from_str(&json)?;
        Ok(profile)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            rough_output_height: "1000".to_string(),
            sensitivity: "100".to_string(),
            scan_step: "5".to_string(),
            edges: "5".to_string(),
            output_format: "jpg".to_string(),
            output_quality: "90".to_string(),
            custom_width_enabled: false,
            custom_width: "".to_string(),
            upscale_enabled: false,
            upscale_factor: 1,
            resize_enabled: false,
            resize_width: "".to_string(),
            resize_height: "".to_string(),
            waifu2x_enabled: false,
            waifu2x_mode: "noise_scale".to_string(),
            waifu2x_noise_level: "1".to_string(),
            waifu2x_scale_mode: "ratio".to_string(),
            waifu2x_scale_ratio: "2.0".to_string(),
            waifu2x_scale_width: "".to_string(),
            waifu2x_scale_height: "".to_string(),
            waifu2x_model: "anime_style_art".to_string(),
            waifu2x_tta: false,
            waifu2x_crop_size: "128".to_string(),
            waifu2x_batch_size: "1".to_string(),
            waifu2x_process: "gpu".to_string(),
            waifu2x_output_depth: "8".to_string(),
        }
    }
} 