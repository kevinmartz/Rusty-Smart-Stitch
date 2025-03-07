#![windows_subsystem = "windows"] // Uncomment or comment this if you don't want a console window or want a console window

mod about_tab;
mod advanced_tab;
mod checkupd;
mod folder_handler;
mod main_tab;
mod process_handler;
mod profile;
mod style;
mod waifu2x_tab;

use eframe::egui::{self};
use egui::IconData;
use egui::ViewportBuilder;
use profile::Profile;
use rusty_smart_stitch::RustySmartStitch;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Instant;

#[derive(PartialEq)]
enum Tab {
    Main,
    Advanced,
    About,
    Waifu2x,
}

#[derive(Debug)]
enum ProcessStatus {
    Progress(f32),
    Waifu2xProgress(usize, usize, String),
    Complete,
    Error(String),
}

#[derive(Debug)]
enum UpdateStatus {
    Complete,
    Error(()),
    Downloading,
    Installing,
}

struct RustySmartStitchApp {
    // Basic file handling
    input_paths: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    manual_output_dir: String,
    root_input_path: Option<PathBuf>,
    root_output_dir: Option<PathBuf>,
    last_output_format: Option<String>, // Track the last used output format

    // Main settings
    rough_output_height: String,
    sensitivity: String,
    scan_step: String,
    edges: String,
    output_format: String,
    output_quality: String,

    // Progress tracking
    progress: f32,
    target_progress: f32,
    processing: bool,
    error_message: String,
    success_message: String,
    profile_message: String,
    progress_rx: Option<Receiver<ProcessStatus>>,
    last_update: Option<Instant>,
    processed_folder_count: usize,

    // UI stuff
    drag_hovering: bool,
    current_tab: Tab,

    // Advanced settings
    custom_width_enabled: bool,
    custom_width: String,
    upscale_enabled: bool,
    upscale_factor: u32,
    resize_enabled: bool,
    resize_width: String,
    resize_height: String,

    // Update checker stuff
    checking_updates: bool,
    update_status_rx: Option<Receiver<UpdateStatus>>,
    current_update_status: Option<UpdateStatus>,

    // UI elements
    drag_icon: Option<egui::TextureHandle>,
    current_progress_message: String,
    last_message_update: Option<Instant>,
    message_transition: f32,

    // Waifu2x settings (yeah there's a lot)
    waifu2x_tta: bool,
    waifu2x_gpu: String,
    waifu2x_batch_size: String,
    waifu2x_crop_h: String,
    waifu2x_crop_w: String,
    waifu2x_crop_size: String,
    waifu2x_output_depth: String,
    waifu2x_process: String,
    waifu2x_model_dir: String,
    waifu2x_scale_height: String,
    waifu2x_scale_width: String,
    waifu2x_scale_ratio: String,
    waifu2x_noise_level: String,
    waifu2x_mode: String,
    waifu2x_exe_path: String,
    waifu2x_enabled: bool,
    waifu2x_scale_mode: String,
    waifu2x_model: String,
    waifu2x_split_mode: String,

    // Random stuff
    random_message: String,
    profiles: Vec<(String, Profile)>,
    current_profile_name: String,
    pending_subfolders: Option<VecDeque<PathBuf>>,
}

impl Default for RustySmartStitchApp {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("rusty_smart_stitch"))
            .unwrap_or_default();

        let waifu2x_exe_path =
            if let Ok(content) = std::fs::read_to_string(config_dir.join("waifu2x_path.txt")) {
                content
            } else {
                String::new()
            };

        // defaults
        let mut app = Self {
            input_paths: Vec::new(),
            output_dir: None,
            manual_output_dir: String::new(),
            root_input_path: None,
            root_output_dir: None,
            last_output_format: None,

            rough_output_height: String::from("1000"),
            sensitivity: String::from("100"),
            scan_step: String::from("5"),
            edges: String::from("5"),
            output_format: String::from("jpg"),
            output_quality: String::from("90"),
            progress: 0.0,
            target_progress: 0.0,
            processing: false,
            error_message: String::new(),
            success_message: String::new(),
            profile_message: String::new(),
            progress_rx: None,
            last_update: None,
            processed_folder_count: 0,
            drag_hovering: false,
            current_tab: Tab::Main, // Start in the main tab, duh
            custom_width_enabled: false,
            custom_width: "0".to_string(),
            upscale_enabled: false,
            upscale_factor: 1,
            resize_enabled: false,
            resize_width: "0".to_string(),
            resize_height: "0".to_string(),
            checking_updates: false,
            update_status_rx: None,
            current_update_status: None,
            drag_icon: None,
            current_progress_message: String::new(),
            last_message_update: None,
            message_transition: 1.0,
            // All the waifu2x defaults - mostly stolen from the original app
            waifu2x_tta: false,
            waifu2x_gpu: "0".to_string(),
            waifu2x_batch_size: "1".to_string(),
            waifu2x_crop_h: "128".to_string(),
            waifu2x_crop_w: "128".to_string(),
            waifu2x_crop_size: "128".to_string(),
            waifu2x_output_depth: "8".to_string(),
            waifu2x_process: "gpu".to_string(),
            waifu2x_model_dir: String::new(),
            waifu2x_scale_height: "0".to_string(),
            waifu2x_scale_width: "0".to_string(),
            waifu2x_scale_ratio: "2.0".to_string(),
            waifu2x_noise_level: "1".to_string(),
            waifu2x_mode: "noise_scale".to_string(),
            waifu2x_exe_path,
            waifu2x_enabled: false,
            waifu2x_scale_mode: "ratio".to_string(),
            waifu2x_model: "anime_style_art".to_string(),
            waifu2x_split_mode: "default".to_string(),
            random_message: String::new(),
            profiles: Vec::new(),
            current_profile_name: String::new(),
            pending_subfolders: None,
        };

        // Try to load any saved profiles
        let _ = app.load_profiles();

        app
    }
}

impl RustySmartStitchApp {
    // Profile management stuff
    fn create_profile(&mut self, name: String) {
        if let Some(index) = self.profiles.iter().position(|(n, _)| n == &name) {
            let profile = Profile {
                rough_output_height: self.rough_output_height.clone(),
                sensitivity: self.sensitivity.clone(),
                scan_step: self.scan_step.clone(),
                edges: self.edges.clone(),
                output_format: self.output_format.clone(),
                output_quality: self.output_quality.clone(),
                custom_width_enabled: self.custom_width_enabled,
                custom_width: self.custom_width.clone(),
                upscale_enabled: self.upscale_enabled,
                upscale_factor: self.upscale_factor,
                resize_enabled: self.resize_enabled,
                resize_width: self.resize_width.clone(),
                resize_height: self.resize_height.clone(),
                waifu2x_enabled: self.waifu2x_enabled,
                waifu2x_mode: self.waifu2x_mode.clone(),
                waifu2x_noise_level: self.waifu2x_noise_level.clone(),
                waifu2x_scale_mode: self.waifu2x_scale_mode.clone(),
                waifu2x_scale_ratio: self.waifu2x_scale_ratio.clone(),
                waifu2x_scale_width: self.waifu2x_scale_width.clone(),
                waifu2x_scale_height: self.waifu2x_scale_height.clone(),
                waifu2x_model: self.waifu2x_model.clone(),
                waifu2x_tta: self.waifu2x_tta,
                waifu2x_crop_size: self.waifu2x_crop_size.clone(),
                waifu2x_batch_size: self.waifu2x_batch_size.clone(),
                waifu2x_process: self.waifu2x_process.clone(),
                waifu2x_output_depth: self.waifu2x_output_depth.clone(),
            };
            self.profiles[index] = (name.clone(), profile);
            self.profile_message = format!("Updated profile: {}", name);
        } else {
            // Make a new profile with current settings
            let profile = Profile {
                rough_output_height: self.rough_output_height.clone(),
                sensitivity: self.sensitivity.clone(),
                scan_step: self.scan_step.clone(),
                edges: self.edges.clone(),
                output_format: self.output_format.clone(),
                output_quality: self.output_quality.clone(),
                custom_width_enabled: self.custom_width_enabled,
                custom_width: self.custom_width.clone(),
                upscale_enabled: self.upscale_enabled,
                upscale_factor: self.upscale_factor,
                resize_enabled: self.resize_enabled,
                resize_width: self.resize_width.clone(),
                resize_height: self.resize_height.clone(),
                waifu2x_enabled: self.waifu2x_enabled,
                waifu2x_mode: self.waifu2x_mode.clone(),
                waifu2x_noise_level: self.waifu2x_noise_level.clone(),
                waifu2x_scale_mode: self.waifu2x_scale_mode.clone(),
                waifu2x_scale_ratio: self.waifu2x_scale_ratio.clone(),
                waifu2x_scale_width: self.waifu2x_scale_width.clone(),
                waifu2x_scale_height: self.waifu2x_scale_height.clone(),
                waifu2x_model: self.waifu2x_model.clone(),
                waifu2x_tta: self.waifu2x_tta,
                waifu2x_crop_size: self.waifu2x_crop_size.clone(),
                waifu2x_batch_size: self.waifu2x_batch_size.clone(),
                waifu2x_process: self.waifu2x_process.clone(),
                waifu2x_output_depth: self.waifu2x_output_depth.clone(),
            };
            self.profiles.push((name.clone(), profile));
            self.profile_message = format!("Created new profile: {}", name);
        }
    }

    // Load settings from a profile
    fn load_profile(&mut self, name: &str) {
        if let Some((_, profile)) = self.profiles.iter().find(|(n, _)| n == name) {
            // Copy all the settings over
            self.rough_output_height = profile.rough_output_height.clone();
            self.sensitivity = profile.sensitivity.clone();
            self.scan_step = profile.scan_step.clone();
            self.output_format = profile.output_format.clone();
            self.output_quality = profile.output_quality.clone();

            // Advanced stuff too
            self.custom_width_enabled = profile.custom_width_enabled;
            self.custom_width = profile.custom_width.clone();
            self.upscale_enabled = profile.upscale_enabled;
            self.upscale_factor = profile.upscale_factor;
            self.resize_enabled = profile.resize_enabled;
            self.resize_width = profile.resize_width.clone();
            self.resize_height = profile.resize_height.clone();

            // And all the waifu2x settings
            self.waifu2x_enabled = profile.waifu2x_enabled;
            self.waifu2x_mode = profile.waifu2x_mode.clone();
            self.waifu2x_noise_level = profile.waifu2x_noise_level.clone();
            self.waifu2x_scale_mode = profile.waifu2x_scale_mode.clone();
            self.waifu2x_scale_ratio = profile.waifu2x_scale_ratio.clone();
            self.waifu2x_scale_width = profile.waifu2x_scale_width.clone();
            self.waifu2x_scale_height = profile.waifu2x_scale_height.clone();
            self.waifu2x_model = profile.waifu2x_model.clone();
            self.waifu2x_tta = profile.waifu2x_tta;
            self.waifu2x_crop_size = profile.waifu2x_crop_size.clone();
            self.waifu2x_batch_size = profile.waifu2x_batch_size.clone();
            self.waifu2x_process = profile.waifu2x_process.clone();
            self.waifu2x_output_depth = profile.waifu2x_output_depth.clone();
        }
    }

    // Save profiles to local
    fn save_profiles(&self) -> Result<(), anyhow::Error> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("rusty_smart_stitch");
            std::fs::create_dir_all(&config_dir)?;
            let profiles_file = config_dir.join("profiles.json");
            let json = serde_json::to_string_pretty(&self.profiles)?;
            std::fs::write(profiles_file, json)?;
        }
        Ok(())
    }

    // Load profiles from local
    fn load_profiles(&mut self) -> Result<(), anyhow::Error> {
        if let Some(config_dir) = dirs::config_dir() {
            let profiles_file = config_dir.join("rusty_smart_stitch").join("profiles.json");
            if profiles_file.exists() {
                let json = std::fs::read_to_string(profiles_file)?;
                self.profiles = serde_json::from_str(&json)?;
            }
        }
        Ok(())
    }

    // Export a profile to a file so you can share it
    fn export_profile(&self, name: &str) -> Result<(), anyhow::Error> {
        if let Some((_, profile)) = self.profiles.iter().find(|(n, _)| n == name) {
            let filename = format!("{}.json", name);
            let file_dialog = native_dialog::FileDialog::new()
                .set_filename(&filename)
                .add_filter("JSON Profile", &["json"]);

            if let Ok(Some(path)) = file_dialog.show_save_single_file() {
                profile.save_to_file(&path)?;
            }
        }
        Ok(())
    }

    // Import a profile from a file
    fn import_profile(&mut self) -> Result<(), anyhow::Error> {
        let file_dialog = native_dialog::FileDialog::new().add_filter("JSON Profile", &["json"]);

        if let Ok(Some(path)) = file_dialog.show_open_single_file() {
            let profile = Profile::load_from_file(&path)?;
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Imported Profile")
                .to_string();
            self.profiles.push((name, profile));
            self.save_profiles()?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Load icon
    let icon = include_bytes!("../assets/icon.png");
    let icon = load_icon(icon);

    let mut options = eframe::NativeOptions::default();
    options.viewport = ViewportBuilder::default()
        .with_inner_size([484.0, 725.0]) // Perfect size
        .with_min_inner_size([484.0, 720.0]) // Don't let it get smaller
        .with_max_inner_size([484.0, 720.0]) // Or bigger
        .with_resizable(false) // Seriously, don't resize it
        .with_maximized(false) // Or maximize it
        .with_maximize_button(false) // Or even try to maximize it
        .with_transparent(false) // No transparency
        .with_decorations(true) // Keep the window border
        .with_icon(icon); // cool icon

    eframe::run_native(
        "Rusty Smart Stitch",
        options,
        Box::new(|cc| {
            // Set up fonts
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "pixel-font".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/Broken-Console-Bold.ttf")),
            );

            // emojis for folder icons and stuff
            fonts.font_data.insert(
                "emoji-font".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/NotoEmoji-Regular.ttf")),
            );

            fonts.font_data.insert(
                "progress-font".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/bold.ttf")),
            );

            fonts
                .families
                .entry(egui::FontFamily::Name("PixelFont".into()))
                .or_default()
                .insert(0, "pixel-font".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Name("ProgressFont".into()))
                .or_default()
                .insert(0, "progress-font".to_owned());

            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push("emoji-font".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(RustySmartStitchApp::default()))
        }),
    )
}

fn load_icon(icon_bytes: &[u8]) -> IconData {
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    IconData {
        rgba,
        width,
        height,
    }
}
