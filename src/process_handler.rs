use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::thread;
use rand::seq::SliceRandom;
use std::process::Command;
use std::path::Path;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::{RustySmartStitchApp, ProcessStatus, RustySmartStitch};
use eframe::egui::{self, Color32, RichText, Rect, Rounding, Stroke, Vec2};
use rusty_smart_stitch::waifu2x::Waifu2xConfig;

// Shit for making the UI look good, but srs this is UI constants basicly you want to change these or comment them and use your own hardcoded values.
const PROGRESS_BAR_COLOR: Color32 = Color32::from_rgb(14, 138, 199);
const BACKGROUND_COLOR: Color32 = Color32::from_gray(20);
const BORDER_STROKE: f32 = 1.0;
const BORDER_COLOR: Color32 = Color32::from_gray(100);
const MESSAGE_UPDATE_INTERVAL: Duration = Duration::from_secs(4);
const PROGRESS_UPDATE_INTERVAL: Duration = Duration::from_millis(16);
const FADE_STEP: f32 = 0.02;
const PROGRESS_THRESHOLD: f32 = 0.0001;
const COMPLETION_THRESHOLD: f32 = 0.97;

impl RustySmartStitchApp {
    pub fn handle_process_status(&mut self, status: ProcessStatus) {
        match status {
            ProcessStatus::Progress(progress) => {
                if let Some(total_folders) = self.get_total_folder_count() {
                    let processed_folders = self.processed_folder_count as f32;
                    let overall_progress = (processed_folders + progress) / (total_folders as f32);
                    self.target_progress = overall_progress.min(1.0);
                } else {
                    self.target_progress = progress;
                }
            }
            ProcessStatus::Waifu2xProgress(current, total, filename) => {
                if let Some(total_folders) = self.get_total_folder_count() {
                    let processed_folders = self.processed_folder_count as f32;
                    let waifu2x_progress = (current as f32 / total as f32) * 0.5;
                    let overall_progress = (processed_folders + 0.5 + waifu2x_progress) / (total_folders as f32);
                    self.target_progress = overall_progress.min(1.0);
                } else {
                    let waifu2x_progress = (current as f32 / total as f32) * 0.5;
                    self.target_progress = (0.5 + waifu2x_progress).min(1.0);
                }
                self.current_progress_message = format!("Applying Waifu2x to: {}", filename);
                if !self.current_progress_message.contains("Waifu2x") {
                    self.last_message_update = None;
                }
            }
            ProcessStatus::Complete => {
                self.complete_processing();
            }
            ProcessStatus::Error(e) => {
                self.handle_error(e);
            }
        }
    }

    // Get total number of folders to process
    fn get_total_folder_count(&self) -> Option<usize> {
        if let Some(ref queue) = self.pending_subfolders {
            Some(queue.len() + self.processed_folder_count + 1)
        } else {
            None
        }
    }

    // checks if there's more folders to process
    fn complete_processing(&mut self) {
        if let Some(ref mut queue) = self.pending_subfolders {
            if !queue.is_empty() {
                self.processed_folder_count += 1;
                self.success_message = format!("Folder processed! Processing next subfolder...").to_uppercase();
                if self.process_next_subfolder() {
                    self.start_processing();
                }
            } else {
                self.processing = false;
                self.progress = 1.0;
                self.target_progress = 1.0;
                self.error_message.clear();
                self.success_message = format!("Successfully processed all folders!").to_uppercase();
                self.progress_rx = None;
                self.last_update = None;
                self.processed_folder_count = 0;
                self.pending_subfolders = None;
            }
        } else {
            self.processing = false;
            self.progress = 1.0;
            self.target_progress = 1.0;
            self.error_message.clear();
            self.success_message = format!("Successfully processed all images!").to_uppercase();
            self.progress_rx = None;
            self.last_update = None;
            self.processed_folder_count = 0;
        }
    }

    fn handle_error(&mut self, error: String) {
        self.processing = false;
        self.error_message = error;
        self.success_message.clear();
        self.progress_rx = None;
        self.pending_subfolders = None;
        self.processed_folder_count = 0;
    }

    pub fn start_processing(&mut self) {
        self.setup_output_directory();
        
        if self.progress_rx.is_none() {
            self.initialize_processing_state();
        }
        
        let processor = self.create_rusty_smart_stitch();
        let (tx, rx) = mpsc::channel();
        
        self.spawn_processing_thread(processor, tx);
        self.progress_rx = Some(rx);
        
        if self.current_progress_message.is_empty() {
            self.set_initial_progress_message();
        }
    }

    fn initialize_processing_state(&mut self) {
        self.processing = true;
        self.error_message.clear();
        self.success_message.clear();
        self.progress = 0.0;
        self.target_progress = 0.0;
        self.last_update = Some(Instant::now());
        self.last_message_update = None;
        self.message_transition = 1.0;
        self.current_progress_message = String::new();
    }

    fn create_rusty_smart_stitch(&self) -> RustySmartStitch {
        let height = self.parse_and_clamp(&self.rough_output_height, 800, 100, 10000);
        let sensitivity = self.parse_and_clamp(&self.sensitivity, 100, 1, 100);
        let scan_step = self.parse_and_clamp(&self.scan_step, 5, 1, 20);
        let quality = self.parse_and_clamp(&self.output_quality, 90, 1, 100);
        let edges = self.parse_and_clamp(&self.edges, 5, 1, 20);

        RustySmartStitch::new(
            self.input_paths.clone(),
            self.output_dir.as_ref().unwrap().clone(),
            height,
            sensitivity,
            scan_step,
            edges,
            self.output_format.clone(),
            quality,
            self.custom_width_enabled,
            self.custom_width.parse::<u32>().unwrap_or(0),
            self.upscale_enabled,
            self.upscale_factor,
            self.resize_enabled,
            self.resize_width.parse::<u32>().unwrap_or(0),
            self.resize_height.parse::<u32>().unwrap_or(0),
        )
    }

    fn parse_and_clamp<T>(&self, value: &str, default: T, min: T, max: T) -> T 
    where
        T: std::str::FromStr + std::cmp::Ord,
    {
        value.parse::<T>().unwrap_or(default).clamp(min, max)
    }

    fn spawn_processing_thread(&self, processor: RustySmartStitch, tx: mpsc::Sender<ProcessStatus>) {
        let waifu2x_enabled = self.waifu2x_enabled;
        let waifu2x_config = waifu2x_enabled.then(|| self.create_waifu2x_config());
        let output_dir = self.output_dir.as_ref().unwrap().clone();
        let output_quality = self.output_quality.clone();

        thread::spawn(move || {
            if let Err(e) = Self::run_processing(processor, waifu2x_enabled, waifu2x_config, &output_dir, &output_quality, &tx) {
                tx.send(ProcessStatus::Error(e.to_string())).unwrap_or_default();
            }
        });
    }

    fn run_processing(
        processor: RustySmartStitch,
        waifu2x_enabled: bool,
        waifu2x_config: Option<Waifu2xConfig>,
        output_dir: &Path,
        output_quality: &str,
        tx: &mpsc::Sender<ProcessStatus>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // First the normal processing
        processor.process_with_progress(|progress| {
            tx.send(ProcessStatus::Progress(progress * 0.5)).unwrap_or_default();
        })?;

        // Then waifu2x if that's enabled
        if waifu2x_enabled {
            if let Some(config) = waifu2x_config {
                // Run waifu2x on the output
                Self::process_with_waifu2x(output_dir, &config, output_quality, tx)?;
            }
        }

        tx.send(ProcessStatus::Complete).unwrap_or_default();
        Ok(())
    }

    // Run waifu2x on all the images
    fn process_with_waifu2x(
        output_dir: &Path,
        config: &Waifu2xConfig,
        output_quality: &str,
        tx: &mpsc::Sender<ProcessStatus>
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !output_dir.exists() {
            return Ok(());
        }

        let files = Self::collect_image_files(output_dir)?;
        let total_files = files.len();

        for (i, entry) in files.into_iter().enumerate() {
            let input_path = entry.path();
            let filename = input_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let cmd = Self::build_waifu2x_command(&input_path, config, output_quality)?;
            
            tx.send(ProcessStatus::Waifu2xProgress(i + 1, total_files, filename))
                .unwrap_or_default();

            Self::run_waifu2x_command(cmd, &input_path)?;
        }

        Ok(())
    }

    // the acutal waifu2x command builder is in waifu2x.rs it uses that as a base
    // Sets up the waifu2x command with all its options
    fn build_waifu2x_command(
        input_path: &Path,
        config: &Waifu2xConfig,
        output_quality: &str,
    ) -> Result<Command, Box<dyn std::error::Error>> {
        let mut cmd = Command::new(&config.executable_path);
        
        if let Some(mode) = &config.mode {
            cmd.arg("-m").arg(mode);
        } else {
            cmd.arg("-m").arg("noise_scale");
        }

        Self::add_optional_arg(&mut cmd, "-n", config.noise_level, "1");
        Self::add_optional_arg(&mut cmd, "-s", config.scale_ratio, "2");
        
        if let Some(model_dir) = &config.model_dir {
            cmd.arg("--model_dir").arg(model_dir);
        }

        // Quality and paths. quality will be set to the output quality from the UI
        cmd.arg("-q").arg(output_quality);
        cmd.arg("-i").arg(input_path);
        
        let output_path = input_path.with_file_name(format!(
            "{}_waifu2x{}",
            input_path.file_stem().unwrap().to_string_lossy(),
            input_path.extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default()
        ));
        cmd.arg("-o").arg(&output_path);

        if let Some(true) = config.tta {
            cmd.arg("-t").arg("1");
        }
        Self::add_optional_arg(&mut cmd, "--gpu", config.gpu, "");
        Self::add_optional_arg(&mut cmd, "-b", config.batch_size, "");
        if let Some(process) = &config.process {
            cmd.arg("-p").arg(process);
        }

        // handle split size, this is not used but it's here just in case if you want to add it in the future. just make sure to update the UI to match
        if let (Some(crop_w), Some(crop_h)) = (config.crop_w, config.crop_h) {
            cmd.arg("--crop_w").arg(crop_w.to_string());
            cmd.arg("--crop_h").arg(crop_h.to_string());
        } else if let Some(crop_size) = config.crop_size {
            cmd.arg("-c").arg(crop_size.to_string());
        }
        // handle output depth, this is only meant to use if you set the output format to png. but it should help with jpgs too kinda?
        if let Some(depth) = config.output_depth {
            if depth != 8 {
                cmd.arg("-d").arg(depth.to_string());
            }
        }

        Ok(cmd)
    }

    fn add_optional_arg<T: std::fmt::Display>(cmd: &mut Command, flag: &str, value: Option<T>, default: &str) {
        match value {
            Some(v) => { cmd.arg(flag).arg(v.to_string()); }
            None if !default.is_empty() => { cmd.arg(flag).arg(default); }
            _ => {}
        }
    }

    // run the waifu2x command
    fn run_waifu2x_command(mut cmd: Command, input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("Executing Waifu2x command: {:?}", cmd);

        let output_path = input_path.with_file_name(format!(
            "{}_waifu2x{}",
            input_path.file_stem().unwrap().to_string_lossy(),
            input_path.extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default()
        ));

        // Hide terminal window on Windows only
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let status = cmd.spawn()?.wait()?;
        if !status.success() {
            return Err(format!("Waifu2x process failed with status: {}", status).into());
        }

        std::fs::rename(&output_path, input_path)?;
        Ok(())
    }

    pub fn draw_progress_area(&mut self, ui: &mut egui::Ui, rect: Rect, available_height: f32) {
        ui.painter().rect_filled(rect, Rounding::same(4.0), BACKGROUND_COLOR);
        
        if self.processing {
            self.draw_processing_state(ui, rect, available_height);
        } else if !self.error_message.is_empty() {
            self.draw_error_state(ui, rect, available_height);
        } else if !self.success_message.is_empty() {
            self.draw_success_state(ui, rect, available_height);
        }
    }

    fn draw_processing_state(&mut self, ui: &mut egui::Ui, rect: Rect, available_height: f32) {
        let progress_width = rect.width() * self.progress;
        
        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(progress_width, rect.height())),
            Rounding::same(4.0),
            PROGRESS_BAR_COLOR,
        );
        ui.painter().rect_stroke(rect, Rounding::same(4.0), Stroke::new(BORDER_STROKE, BORDER_COLOR));

        self.draw_progress_message(ui, available_height);
        self.draw_random_message(ui);
    }

    fn draw_progress_message(&self, ui: &mut egui::Ui, available_height: f32) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(if self.progress > 0.5 && self.waifu2x_enabled {
                available_height / 2.0 - 20.0
            } else {
                available_height / 3.0 - 20.0
            });

            if self.progress > 0.5 && self.waifu2x_enabled {
                ui.label(
                    RichText::new(&self.current_progress_message.to_uppercase())
                        .color(Color32::WHITE)
                        .size(21.0)
                        .family(egui::FontFamily::Name("ProgressFont".into()))
                );
            } else {
                ui.label(
                    RichText::new(format!("{:.1}%", self.progress * 100.0))
                        .color(Color32::WHITE)
                        .size(22.0)
                        .family(egui::FontFamily::Name("PixelFont".into()))
                );
            }
        });
    }

    fn draw_random_message(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new(&self.random_message)
                    .color(Color32::from_rgb(255, 255, 255))
                    .size(15.0)
                    .family(egui::FontFamily::Name("ProgressFont".into()))
            );
        });
    }

    fn draw_success_state(&self, ui: &mut egui::Ui, rect: Rect, available_height: f32) {
        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(BORDER_STROKE, BORDER_COLOR));
        
        ui.vertical_centered_justified(|ui| {
            ui.add_space(available_height / 2.0 - 10.0);
            ui.label(
                RichText::new(&self.success_message)
                    .color(PROGRESS_BAR_COLOR)
                    .size(10.0)
                    .family(egui::FontFamily::Name("PixelFont".into()))
            );
        });
    }

    fn draw_error_state(&self, ui: &mut egui::Ui, rect: Rect, available_height: f32) {
        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(BORDER_STROKE, Color32::from_rgb(255, 100, 100)));
        
        ui.vertical_centered_justified(|ui| {
            ui.add_space(available_height / 2.0 - 10.0);
            ui.label(
                RichText::new(&self.error_message)
                    .color(Color32::from_rgb(255, 100, 100))
                    .size(15.0)
                    .family(egui::FontFamily::Name("ProgressFont".into()))
            );
        });
    }

    pub fn update_progress(&mut self, ctx: &egui::Context) {
        self.check_progress_receiver();
        if self.processing {
            self.update_messages();
            self.update_progress_smoothly(ctx);
            ctx.request_repaint();
        }
    }

    fn check_progress_receiver(&mut self) {
        if let Some(rx) = &self.progress_rx {
            if let Ok(status) = rx.try_recv() {
                self.handle_process_status(status);
            }
        }
    }

    fn update_messages(&mut self) {
        let now = Instant::now();
        
        if self.last_message_update.is_none() {
            self.initialize_random_message(now);
        } else if let Some(last_update) = self.last_message_update {
            self.update_message_transition(last_update, now);
        }
    }

    fn initialize_random_message(&mut self, now: Instant) {
        self.random_message = self.choose_random_message();
        self.last_message_update = Some(now);
        self.message_transition = 1.0;
    }

    fn choose_random_message(&self) -> String {
        if self.progress > 0.5 && self.waifu2x_enabled {
            WAIFU2X_MESSAGES
        } else {
            PROGRESS_MESSAGES
        }.choose(&mut rand::thread_rng())
            .unwrap_or(&"Processing...")
            .to_string()
    }

    fn update_message_transition(&mut self, last_update: Instant, now: Instant) {
        if last_update.elapsed() > MESSAGE_UPDATE_INTERVAL {
            self.message_transition = (self.message_transition - FADE_STEP).max(0.0);
            
            if self.message_transition <= 0.0 {
                self.random_message = self.choose_random_message();
                self.last_message_update = Some(now);
                self.message_transition = 1.0;
            }
        } else if self.message_transition < 1.0 {
            self.message_transition = (self.message_transition + FADE_STEP).min(1.0);
        }
    }

    fn update_progress_smoothly(&mut self, ctx: &egui::Context) {
        if let Some(last_update) = self.last_update {
            if last_update.elapsed() > PROGRESS_UPDATE_INTERVAL {
                if (self.target_progress - self.progress).abs() > PROGRESS_THRESHOLD {
                    self.progress += (self.target_progress - self.progress) * 0.1;
                    ctx.request_repaint();
                } else if self.target_progress >= COMPLETION_THRESHOLD {
                    self.progress = self.target_progress;
                }
                self.last_update = Some(Instant::now());
            }
        } else {
            self.last_update = Some(Instant::now());
        }
    }

    fn set_initial_progress_message(&mut self) {
        self.current_progress_message = PROGRESS_MESSAGES
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"Processing...")
            .to_string();
    }

    fn create_waifu2x_config(&self) -> Waifu2xConfig {
        let mut config = Waifu2xConfig::default();
        

        config.executable_path = self.waifu2x_exe_path.clone();
        
        config.model = self.waifu2x_model.clone();
        config.model_dir = Some(format!("models/{}", self.waifu2x_model));
        
        if self.waifu2x_tta {
            config.tta = Some(true);
        }
        
        if let Ok(gpu) = self.waifu2x_gpu.parse() {
            config.gpu = Some(gpu);
        }
        
        if let Ok(batch_size) = self.waifu2x_batch_size.parse() {
            config.batch_size = Some(batch_size);
        }
        
        if self.waifu2x_split_mode == "custom" {
            if let (Ok(crop_w), Ok(crop_h)) = (self.waifu2x_crop_w.parse(), self.waifu2x_crop_h.parse()) {
                config.crop_w = Some(crop_w);
                config.crop_h = Some(crop_h);
            }
        } else {
            config.crop_size = Some(128);
        }
        
        if let Ok(output_depth) = self.waifu2x_output_depth.parse() {
            config.output_depth = Some(output_depth);
        }
        
        config.process = Some(self.waifu2x_process.clone());
        
        if !self.waifu2x_model_dir.is_empty() {
            config.model_dir = Some(self.waifu2x_model_dir.clone());
        }
        
        if let Ok(scale_height) = self.waifu2x_scale_height.parse() {
            config.scale_height = Some(scale_height);
        }
        
        if let Ok(scale_width) = self.waifu2x_scale_width.parse() {
            config.scale_width = Some(scale_width);
        }
        
        if let Ok(scale_ratio) = self.waifu2x_scale_ratio.parse() {
            config.scale_ratio = Some(scale_ratio);
        }
        
        if let Ok(noise_level) = self.waifu2x_noise_level.parse() {
            config.noise_level = Some(noise_level);
        }
        
        config.mode = Some(self.waifu2x_mode.clone());
        
        // Uses the main output format
        config.output_extension = Some(self.output_format.clone());
        
        config
    }
}

const PROGRESS_MESSAGES: &[&str] = &[
    "Processing... promise it's not magic. Or is it?",
    "Aligning pixels... hope they get along!",
    "Sewing images together... one stitch at a time!",
    "Making a digital quilt... almost done!",
    "Convincing edges to hold hands...",
    "Lining things up... this is not Tetris, promise!",
    "Connecting the dots... but with way more pixels!",
    "Pixel matchmaking in progress... true love takes time!",
    "Gathering corners for a meeting... almost adjourned!",
    "Telling photos to stop arguing...",
    "Chopping off the unneeded bits... like a pixel barber!",
    "Resizing reality... digitally, of course.",
    "Giving images a trim... scissors not required!",
    "Cropping corners (but only the right ones)!",
    "Putting the 'pro' in progress!",
    "Loading... pixels are unionized and on a break.",
];

const WAIFU2X_MESSAGES: &[&str] = &[
    "Making pixels bigger... but prettier!",
    "Zoom, enhance... but for real this time!",
    "That waifu is leveling up... almost there!",
    "Enlarging with style... not just Ctrl+Plus!",
    "Size matters, but quality matters more!",
    "From tiny to shiny... upscaling magic!",
    "Making that image HD... Waifu-approved!",
    "Pixel steroids in action... totally legal!",
    "Stretching pixels... hope they don't snap!",
    "Shushing noisy pixels... quiet, please!",
    "Taking out the trash... noise edition!",
    "Sweeping up artifacts... cleaner than that desk!",
    "De-noising like a vacuum cleaner for pixels!",
    "Silencing rogue pixels... no rebels allowed!",
    "Polishing that image... like a pixel car wash!",
    "Noise? What noise? Never heard of it!",
    "Enhancing... waifus deserve the best!",
    "Doing AI magic... even waifus are impressed!",
    "Making those pixels proud of themselves!",
    "AI upscaling... because regular upscaling is boring!",
    "Cooking that image... medium-rare pixels coming up!",
    "Waifu-level perfection in progress... hold tight!",
    "Quality check... that waifu is in good hands!",
    "Upscaling that waifu... anime magic detected!",
    "Making images shine brighter than that future!",
];