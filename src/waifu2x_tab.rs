use crate::RustySmartStitchApp;
use eframe::egui::{self, Align, Color32, Layout, RichText, Ui, Vec2};
use std::path::Path;

impl RustySmartStitchApp {
    fn save_waifu2x_path(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("rusty_smart_stitch");
            let _ = std::fs::create_dir_all(&config_dir);
            let _ = std::fs::write(config_dir.join("waifu2x_path.txt"), &self.waifu2x_exe_path);
        }
    }

    pub fn show_waifu2x(&mut self, ui: &mut Ui) {
        self.show_waifu2x_header(ui);
        self.show_waifu2x_main_panel(ui);
    }

    fn show_waifu2x_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(
                RichText::new("Waifu2x Settings")
                    .size(24.0)
                    .color(Color32::from_rgb(217, 217, 217)),
            );
            ui.add_space(20.0);
        });
    }

    fn show_waifu2x_main_panel(&mut self, ui: &mut Ui) {
        let panel_width = ui.available_width() - 32.0;

        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
            ui.set_width(panel_width);
            ui.group(|ui| {
                self.show_waifu2x_enable_toggle(ui);
                ui.add_space(10.0);
                self.show_waifu2x_executable_path(ui);
                ui.add_space(10.0);
                self.show_waifu2x_settings(ui);
            });
        });
    }

    fn show_waifu2x_enable_toggle(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let can_enable = Path::new(&self.waifu2x_exe_path).exists();
            let response = ui.add_enabled(
                can_enable,
                egui::Checkbox::new(
                    &mut self.waifu2x_enabled,
                    RichText::new("Enable Waifu2x Processing")
                        .size(16.0)
                        .color(if can_enable {
                            Color32::from_rgb(255, 255, 255)
                        } else {
                            Color32::from_rgb(180, 180, 180)
                        }),
                ),
            );
            if !can_enable {
                response.on_hover_text("Please select waifu2x-caffe-cui.exe first");
            }
        });
    }

    fn show_waifu2x_executable_path(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Waifu2x Executable:").size(14.0));
            if ui.small_button("📁").clicked() {
                if let Ok(Some(path)) = native_dialog::FileDialog::new()
                    .add_filter("Executable", &["exe"])
                    .show_open_single_file()
                {
                    self.waifu2x_exe_path = path.to_string_lossy().to_string();
                    self.save_waifu2x_path();
                }
            }
            ui.add_space(5.0);
            if Path::new(&self.waifu2x_exe_path).exists() {
                ui.label(
                    RichText::new("waifu2x-caffe-cui.exe added successfully!")
                        .size(14.0)
                        .color(Color32::from_rgb(14, 138, 199)),
                );
            } else {
                ui.label(
                    RichText::new("⚠️ Please select waifu2x-caffe-cui.exe")
                        .size(14.0)
                        .color(Color32::from_rgb(255, 180, 0)),
                );
            }
        });
    }

    fn show_waifu2x_settings(&mut self, ui: &mut Ui) {
        ui.add_enabled_ui(self.waifu2x_enabled, |ui| {
            self.show_conversion_mode(ui);
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                self.show_denoise_settings(ui);
                ui.add_space(20.0);
                self.show_magnification_settings(ui);
            });

            ui.add_space(10.0);
            self.show_model_settings(ui);
            ui.add_space(10.0);
            self.show_processing_settings(ui);
        });
    }

    fn show_conversion_mode(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Conversion Mode:").size(14.0));
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.radio_value(
                    &mut self.waifu2x_mode,
                    "noise_scale".to_string(),
                    "Denoise & Magnify",
                );
                ui.radio_value(&mut self.waifu2x_mode, "scale".to_string(), "Magnify only");
                ui.radio_value(&mut self.waifu2x_mode, "noise".to_string(), "Denoise only");
                ui.radio_value(
                    &mut self.waifu2x_mode,
                    "auto_scale".to_string(),
                    "Magnify & Auto Denoise",
                );
            });
        });
    }

    fn show_denoise_settings(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Denoise Level:").size(14.0));
            ui.add_space(5.0);
            let denoise_enabled = self.waifu2x_mode != "scale";
            ui.add_enabled_ui(denoise_enabled, |ui| {
                ui.vertical(|ui| {
                    for level in ["0", "1", "2", "3"] {
                        ui.radio_value(
                            &mut self.waifu2x_noise_level,
                            level.to_string(),
                            format!("Level {}", level),
                        );
                    }
                });
            });
        });
    }

    fn show_magnification_settings(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Magnification Size:").size(14.0));
            ui.add_space(5.0);
            let magnify_enabled = self.waifu2x_mode != "noise";
            ui.add_enabled_ui(magnify_enabled, |ui| {
                self.show_scale_options(ui);
            });
        });
    }

    fn show_scale_options(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let input_width = 70.0;

            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut self.waifu2x_scale_mode,
                    "ratio".to_string(),
                    "Set rate",
                );
                ui.add_space(5.0);
                ui.add_enabled(
                    self.waifu2x_scale_mode == "ratio",
                    egui::TextEdit::singleline(&mut self.waifu2x_scale_ratio)
                        .desired_width(input_width)
                        .hint_text("2.0"),
                );
            });

            self.show_dimension_inputs(ui, input_width);
        });
    }

    fn show_dimension_inputs(&mut self, ui: &mut Ui, input_width: f32) {
        for (mode, label, field) in [
            ("width", "Set width", &mut self.waifu2x_scale_width),
            ("height", "Set height", &mut self.waifu2x_scale_height),
        ] {
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.waifu2x_scale_mode, mode.to_string(), label);
                ui.add_space(5.0);
                ui.add_enabled(
                    self.waifu2x_scale_mode == mode,
                    egui::TextEdit::singleline(field)
                        .desired_width(input_width)
                        .hint_text("0"),
                );
            });
        }

        self.show_both_dimensions(ui, input_width);
    }

    fn show_both_dimensions(&mut self, ui: &mut Ui, input_width: f32) {
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.waifu2x_scale_mode,
                "both".to_string(),
                "Set width & height",
            );
            ui.add_space(5.0);
            if self.waifu2x_scale_mode == "both" {
                ui.add_sized(
                    Vec2::new(input_width, 20.0),
                    egui::TextEdit::singleline(&mut self.waifu2x_scale_width).hint_text("width"),
                );
                ui.label("x");
                ui.add_sized(
                    Vec2::new(input_width, 20.0),
                    egui::TextEdit::singleline(&mut self.waifu2x_scale_height).hint_text("height"),
                );
            }
        });
    }

    fn show_model_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Model:").size(14.0));
            ui.add_space(10.0);
            egui::ComboBox::from_label("")
                .selected_text(&self.waifu2x_model)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.waifu2x_model,
                        "anime_style_art_rgb".to_string(),
                        "anime_style_art_rgb",
                    );
                    ui.selectable_value(
                        &mut self.waifu2x_model,
                        "anime_style_art".to_string(),
                        "anime_style_art",
                    );
                    ui.selectable_value(&mut self.waifu2x_model, "photo".to_string(), "photo");
                    ui.selectable_value(
                        &mut self.waifu2x_model,
                        "upconv_7_anime_style_art_rgb".to_string(),
                        "upconv_7_anime_style_art_rgb",
                    );
                    ui.selectable_value(
                        &mut self.waifu2x_model,
                        "upconv_7_photo".to_string(),
                        "upconv_7_photo",
                    );
                    ui.selectable_value(
                        &mut self.waifu2x_model,
                        "upresnet10".to_string(),
                        "upresnet10",
                    );
                    ui.selectable_value(&mut self.waifu2x_model, "cunet".to_string(), "cunet");
                    ui.selectable_value(&mut self.waifu2x_model, "ukbench".to_string(), "ukbench");
                });

            ui.add_space(20.0);
            ui.checkbox(
                &mut self.waifu2x_tta,
                RichText::new("Use TTA Mode").size(14.0),
            )
            .on_hover_text("8x slower but slightly higher quality");
        });
    }

    fn show_processing_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.show_batch_size(ui);
            ui.add_space(20.0);
            self.show_process_mode(ui);
            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("GPU Device:").size(14.0));
                ui.add_sized(
                    Vec2::new(70.0, 20.0),
                    egui::TextEdit::singleline(&mut self.waifu2x_gpu).hint_text("0"),
                );
            });
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            self.show_split_size(ui);
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Custom Model Dir:").size(14.0));
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(200.0, 20.0),
                        egui::TextEdit::singleline(&mut self.waifu2x_model_dir)
                            .hint_text("models/custom_model"),
                    );
                    if ui.small_button("📁").clicked() {
                        if let Ok(Some(path)) =
                            native_dialog::FileDialog::new().show_open_single_dir()
                        {
                            self.waifu2x_model_dir = path.to_string_lossy().to_string();
                        }
                    }
                });
            });
            ui.add_space(30.0);
            self.show_output_depth(ui);
        });
    }

    fn show_split_size(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Split Size:").size(14.0));
            let row_height = 20.0;
            ui.horizontal(|ui| {
                ui.set_min_height(row_height);
                ui.radio_value(
                    &mut self.waifu2x_split_mode,
                    "default".to_string(),
                    "Default",
                )
                .on_hover_text("Default split size (128x128)");
                ui.add_space(10.0);
                ui.radio_value(&mut self.waifu2x_split_mode, "custom".to_string(), "Custom");

                if self.waifu2x_split_mode == "custom" {
                    ui.add_space(20.0);
                    ui.add_sized(
                        Vec2::new(70.0, row_height),
                        egui::TextEdit::singleline(&mut self.waifu2x_crop_w).hint_text("Width"),
                    );
                    ui.label("x");
                    ui.add_sized(
                        Vec2::new(70.0, row_height),
                        egui::TextEdit::singleline(&mut self.waifu2x_crop_h).hint_text("Height"),
                    );
                }
            });
        });
    }

    fn show_batch_size(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Batch Size:").size(14.0));
            ui.add_sized(
                Vec2::new(70.0, 20.0),
                egui::TextEdit::singleline(&mut self.waifu2x_batch_size).hint_text("1"),
            );
        });
    }

    fn show_process_mode(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Process:").size(14.0));
            ui.horizontal(|ui| {
                for mode in ["CPU", "GPU", "CUDNN"] {
                    let mode_lower = mode.to_lowercase();
                    ui.radio_value(&mut self.waifu2x_process, mode_lower, mode);
                    ui.add_space(5.0);
                }
            });
        });
    }

    fn show_output_depth(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Output Depth:").size(14.0));
            ui.horizontal(|ui| {
                ui.add_sized(
                    Vec2::new(50.0, 20.0),
                    egui::TextEdit::singleline(&mut self.waifu2x_output_depth).hint_text("8"),
                );
                ui.label("bit");
            });
        });
    }

    pub fn has_active_waifu2x(&self) -> bool {
        self.waifu2x_enabled && Path::new(&self.waifu2x_exe_path).exists()
    }
}
