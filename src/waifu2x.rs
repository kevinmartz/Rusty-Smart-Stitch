use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Waifu2xConfig {
    pub input_path: String,
    pub output_path: Option<String>,
    pub executable_path: String,
    pub model: String,
    pub tta: Option<bool>,
    pub gpu: Option<i32>,
    pub batch_size: Option<i32>,
    pub crop_h: Option<i32>,
    pub crop_w: Option<i32>,
    pub crop_size: Option<i32>,
    pub output_depth: Option<i32>,
    pub output_quality: Option<i32>,
    pub process: Option<String>,
    pub model_dir: Option<String>,
    pub scale_height: Option<f64>,
    pub scale_width: Option<f64>,
    pub scale_ratio: Option<f64>,
    pub noise_level: Option<i32>,
    pub mode: Option<String>,
    pub output_extension: Option<String>,
    pub input_extension_list: Option<String>,
}

impl Default for Waifu2xConfig {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: None,
            executable_path: String::new(),
            model: "anime_style_art_rgb".to_string(),
            tta: None,
            gpu: None,
            batch_size: None,
            crop_h: None,
            crop_w: None,
            crop_size: None,
            output_depth: None,
            output_quality: None,
            process: None,
            model_dir: None,
            scale_height: None,
            scale_width: None,
            scale_ratio: None,
            noise_level: None,
            mode: None,
            output_extension: None,
            input_extension_list: None,
        }
    }
}

impl Waifu2xConfig {
    pub fn new(input_path: String) -> Self {
        Self {
            input_path,
            ..Default::default()
        }
    }

    pub fn build_command(&self) -> Result<Command> {
        let mut cmd = Command::new(&self.executable_path);

        cmd.arg("-i").arg(&self.input_path);

        if let Some(output_path) = &self.output_path {
            cmd.arg("-o").arg(output_path);
        }

        if let Some(tta) = self.tta {
            cmd.arg("-t").arg(if tta { "1" } else { "0" });
        }

        if let Some(gpu) = self.gpu {
            cmd.arg("--gpu").arg(gpu.to_string());
        }

        if let Some(batch_size) = self.batch_size {
            cmd.arg("-b").arg(batch_size.to_string());
        }

        if let Some(crop_h) = self.crop_h {
            cmd.arg("--crop_h").arg(crop_h.to_string());
        }

        if let Some(crop_w) = self.crop_w {
            cmd.arg("--crop_w").arg(crop_w.to_string());
        }

        if let Some(crop_size) = self.crop_size {
            cmd.arg("-c").arg(crop_size.to_string());
        }

        if let Some(output_depth) = self.output_depth {
            cmd.arg("-d").arg(output_depth.to_string());
        }

        if let Some(output_quality) = self.output_quality {
            cmd.arg("-q").arg(output_quality.to_string());
        }

        if let Some(process) = &self.process {
            cmd.arg("-p").arg(process);
        }

        if let Some(model_dir) = &self.model_dir {
            cmd.arg("--model_dir").arg(model_dir);
        }

        if let Some(scale_height) = self.scale_height {
            cmd.arg("-h").arg(scale_height.to_string());
        }

        if let Some(scale_width) = self.scale_width {
            cmd.arg("-w").arg(scale_width.to_string());
        }

        if let Some(scale_ratio) = self.scale_ratio {
            cmd.arg("-s").arg(scale_ratio.to_string());
        }

        if let Some(noise_level) = self.noise_level {
            cmd.arg("-n").arg(noise_level.to_string());
        }

        if let Some(mode) = &self.mode {
            cmd.arg("-m").arg(mode);
        }

        if let Some(output_extension) = &self.output_extension {
            cmd.arg("-e").arg(output_extension);
        }

        if let Some(input_extension_list) = &self.input_extension_list {
            cmd.arg("-l").arg(input_extension_list);
        }

        Ok(cmd)
    }

    pub fn run(&self) -> Result<()> {
        let mut cmd = self.build_command()?;
        cmd.spawn()?.wait()?;
        Ok(())
    }
}
