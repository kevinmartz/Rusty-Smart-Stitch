use eframe::egui::{self, Align, Color32, Layout, RichText, Ui, Vec2};

use crate::RustySmartStitchApp;

impl RustySmartStitchApp {
    pub fn show_advanced(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(
                RichText::new("Advanced Settings")
                    .size(24.0)
                    .color(Color32::from_rgb(217, 217, 217)),
            );
            ui.add_space(20.0);
        });

        let panel_width = ui.available_width() - 32.0;

        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
            ui.set_width(panel_width);

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Advanced Settings")
                            .size(18.0)
                            .color(Color32::from_rgb(200, 200, 200)),
                    );
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Custom Width:").size(14.0));
                        ui.add_space(5.0);
                        ui.checkbox(&mut self.custom_width_enabled, "");
                        ui.add_enabled(
                            self.custom_width_enabled,
                            egui::TextEdit::singleline(&mut self.custom_width)
                                .desired_width(70.0)
                                .hint_text("pixels"),
                        )
                        .on_hover_text("Set a custom width for the output image");
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Edges:").size(14.0));
                        ui.add_space(5.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.edges)
                                .desired_width(70.0)
                                .hint_text("1-20"),
                        )
                        .on_hover_text(
                            "Number of pixels to ignore at edges when detecting slice points",
                        );
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Upscale:").size(14.0));
                        ui.add_space(5.0);
                        ui.checkbox(&mut self.upscale_enabled, "");
                        ui.add_enabled_ui(self.upscale_enabled, |ui| {
                            ui.horizontal(|ui| {
                                for factor in [1, 2, 3] {
                                    ui.radio_value(
                                        &mut self.upscale_factor,
                                        factor,
                                        format!("x{}", factor),
                                    );
                                }
                            });
                        })
                        .response
                        .on_hover_text("Scale the image by a fixed multiplier");
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Resize:").size(14.0));
                        ui.add_space(5.0);
                        ui.checkbox(&mut self.resize_enabled, "");
                        ui.add_enabled_ui(self.resize_enabled, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    Vec2::new(70.0, 20.0),
                                    egui::TextEdit::singleline(&mut self.resize_width)
                                        .hint_text("width"),
                                )
                                .on_hover_text("Target width in pixels");

                                ui.label("x");

                                ui.add_sized(
                                    Vec2::new(70.0, 20.0),
                                    egui::TextEdit::singleline(&mut self.resize_height)
                                        .hint_text("height"),
                                )
                                .on_hover_text("Target height in pixels");
                            });
                        });
                    });
                });
            });

            ui.add_space(10.0);

            // Profile management section
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.set_width(panel_width * 0.7);
                    ui.vertical(|ui| {
                        ui.heading(
                            RichText::new("Profile Management")
                                .size(16.0)
                                .color(Color32::from_rgb(200, 200, 200)),
                        );
                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Profile Name:").size(14.0));
                            ui.add_space(5.0);
                            ui.add_sized(
                                Vec2::new(150.0, 20.0),
                                egui::TextEdit::singleline(&mut self.current_profile_name)
                                    .hint_text("Enter profile name"),
                            );

                            ui.add_space(10.0);
                            if ui
                                .add_sized(Vec2::new(80.0, 24.0), egui::Button::new("💾 Save"))
                                .clicked()
                                && !self.current_profile_name.is_empty()
                            {
                                self.create_profile(self.current_profile_name.clone());
                                let _ = self.save_profiles();
                            }
                        });

                        ui.add_space(10.0);

                        // Pick a profile to load
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Load Profile:").size(14.0));
                            ui.add_space(5.0);
                            egui::ComboBox::new("profile_select", "Select a profile")
                                .selected_text(if self.current_profile_name.is_empty() {
                                    "Select a profile"
                                } else {
                                    &self.current_profile_name
                                })
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    for (name, _) in &self.profiles {
                                        ui.selectable_value(
                                            &mut self.current_profile_name,
                                            name.clone(),
                                            name,
                                        );
                                    }
                                });
                        });

                        ui.add_space(15.0);

                        // delete and load profile
                        ui.horizontal(|ui| {
                            ui.add_space(30.0);
                            let profile_name = self.current_profile_name.clone();
                            if !profile_name.is_empty() {
                                if ui
                                    .add_sized(Vec2::new(80.0, 24.0), egui::Button::new("📂 Load"))
                                    .clicked()
                                {
                                    self.load_profile(&profile_name);
                                }
                                ui.add_space(8.0);
                                if ui
                                    .add_sized(
                                        Vec2::new(80.0, 24.0),
                                        egui::Button::new("🗑️ Delete"),
                                    )
                                    .clicked()
                                {
                                    if let Some(pos) =
                                        self.profiles.iter().position(|(n, _)| n == &profile_name)
                                    {
                                        self.profiles.remove(pos);
                                        let _ = self.save_profiles();
                                        self.current_profile_name.clear();
                                        self.profile_message =
                                            format!("Deleted profile: {}", profile_name);
                                    }
                                }
                                ui.add_space(8.0);
                                if ui
                                    .add_sized(
                                        Vec2::new(80.0, 24.0),
                                        egui::Button::new("📤 Export"),
                                    )
                                    .clicked()
                                {
                                    let _ = self.export_profile(&profile_name);
                                }
                            }
                            if !profile_name.is_empty() {
                                ui.add_space(8.0);
                            }
                            if ui
                                .add_sized(Vec2::new(80.0, 24.0), egui::Button::new("📥 Import"))
                                .clicked()
                            {
                                let _ = self.import_profile();
                            }
                        });

                        // Show if something happened with a profile
                        if !self.profile_message.is_empty() {
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&self.profile_message)
                                        .size(12.0)
                                        .color(Color32::from_rgb(100, 200, 100)),
                                );
                            });
                        }
                    });
                });
            });
        });
    }

    // Check if any of that advanced shit is turned on
    pub fn has_active_advanced_settings(&self) -> bool {
        // Return true if any advanced setting is enabled and configured
        (self.custom_width_enabled && !self.custom_width.is_empty())
            || (self.upscale_enabled && self.upscale_factor > 1)
            || (self.resize_enabled
                && (!self.resize_width.is_empty() || !self.resize_height.is_empty()))
    }
}
