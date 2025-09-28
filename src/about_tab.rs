use crate::checkupd::Updater;
use crate::RustySmartStitchApp;
use crate::UpdateStatus;
use eframe::egui::{self, Color32, Frame, RichText, Rounding, Stroke, Ui, Vec2};
use std::sync::mpsc::{self, Receiver, Sender};
use webbrowser;

impl RustySmartStitchApp {
    fn open_url(url: &str) {
        if let Err(e) = webbrowser::open(url) {
            eprintln!("Failed to open URL: {}", e);
        }
    }

    fn card_frame() -> Frame {
        Frame::none()
            .fill(Color32::from_rgb(39, 39, 42))
            .stroke(Stroke::new(1.0, Color32::from_rgb(63, 63, 70)))
            .rounding(Rounding::same(6.0))
            .outer_margin(8.0)
            .inner_margin(8.0)
    }

    fn card_header_frame() -> Frame {
        Frame::none()
            .fill(Color32::from_rgba_premultiplied(39, 39, 42, 200))
            .rounding(Rounding::same(6.0))
            .outer_margin(4.0)
            .inner_margin(8.0)
    }

    pub fn show_about(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(12.0);
            ui.heading(
                RichText::new("Rusty Smart Stitch")
                    .size(24.0)
                    .color(Color32::from_rgb(14, 138, 199))
                    .family(egui::FontFamily::Name("PixelFont".into())),
            );
            ui.label(
                RichText::new("A smart stitch made in Rust for efficient and reliable stitching")
                    .size(14.0)
                    .color(Color32::from_rgb(161, 161, 170)),
            );
            ui.add_space(12.0);

            Self::card_frame().show(ui, |ui| {
                Self::card_header_frame().show(ui, |ui| {
                    ui.heading(
                        RichText::new("Project Info")
                            .size(16.0)
                            .color(Color32::from_rgb(14, 138, 199)),
                    );
                });
                ui.add_space(4.0);

                let table_width = ui.available_width();
                egui::Grid::new("project_info_grid")
                    .spacing(Vec2::new(table_width * 0.5, 4.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("Developer")
                                .size(14.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                        ui.label(RichText::new("Regis").size(14.0).strong());
                        ui.end_row();

                        ui.label(
                            RichText::new("License")
                                .size(14.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                        ui.label(RichText::new("MIT License").size(14.0));
                        ui.end_row();

                        ui.label(
                            RichText::new("Build Date")
                                .size(14.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                        ui.label(RichText::new("9/08/2025").size(14.0));
                        ui.end_row();

                        ui.label(
                            RichText::new("Version")
                                .size(14.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                        ui.label(RichText::new("1.1.2").size(14.0));
                        ui.end_row();
                    });
            });
            ui.add_space(8.0);

            Self::card_frame().show(ui, |ui| {
                Self::card_header_frame().show(ui, |ui| {
                    ui.heading(
                        RichText::new("Description")
                            .size(16.0)
                            .color(Color32::from_rgb(14, 138, 199)),
                    );
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "Rusty Smart Stitch is a stitching solution built with Rust. \
                        It combines the power of smart algorithms with the reliability of Rust, \
                        offering a fast, efficient stitching experience.",
                    )
                    .size(13.0)
                    .color(Color32::from_rgb(212, 212, 216)),
                );
            });
            ui.add_space(8.0);

            if self.checking_updates {
                let status_text = match &self.current_update_status {
                    Some(UpdateStatus::Downloading) => "Downloading update...",
                    Some(UpdateStatus::Installing) => "Installing update...",
                    Some(UpdateStatus::Complete) => {
                        self.checking_updates = false;
                        "Update check complete"
                    }
                    Some(UpdateStatus::Error(_)) => "Update check failed",
                    None => "Checking for updates...",
                };

                ui.label(
                    RichText::new(status_text)
                        .color(Color32::from_rgb(255, 255, 255))
                        .size(14.0),
                );
            } else {
                if ui
                    .button(
                        RichText::new("🔄 Check for Updates")
                            .size(14.0)
                            .color(Color32::from_rgb(212, 212, 216)),
                    )
                    .clicked()
                {
                    self.checking_updates = true;
                    let updater = Updater::new().expect("Failed to initialize updater");
                    let (tx, rx): (Sender<UpdateStatus>, Receiver<UpdateStatus>) = mpsc::channel();
                    self.update_status_rx = Some(rx);
                    self.current_update_status = None;

                    tokio::spawn(async move {
                        match updater.check_for_updates().await {
                            Ok(Some(release)) => {
                                if confirm_update_dialog(&release) {
                                    tx.send(UpdateStatus::Downloading).unwrap_or_default();
                                    match updater.download_update(&release).await {
                                        Ok(new_binary_path) => {
                                            tx.send(UpdateStatus::Installing).unwrap_or_default();
                                            match updater.apply_update(new_binary_path).await {
                                                Ok(_) => (),
                                                Err(e) => {
                                                    eprintln!("Update failed: {}", e);
                                                    show_info_dialog(
                                                        &format!("Update failed: {}", e),
                                                    );
                                                    tx.send(UpdateStatus::Error(()))
                                                        .unwrap_or_default();
                                                    let tx_clone = tx.clone();
                                                    tokio::spawn(async move {
                                                        tokio::time::sleep(
                                                            tokio::time::Duration::from_secs(2),
                                                        )
                                                        .await;
                                                        tx_clone
                                                            .send(UpdateStatus::Complete)
                                                            .unwrap_or_default();
                                                    });
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Download failed: {}", e);
                                            show_info_dialog(
                                                &format!("Download failed: {}. Please try again later or download manually from GitHub.", e),
                                            );
                                            tx.send(UpdateStatus::Error(())).unwrap_or_default();
                                            let tx_clone = tx.clone();
                                            tokio::spawn(async move {
                                                tokio::time::sleep(
                                                    tokio::time::Duration::from_secs(2),
                                                )
                                                .await;
                                                tx_clone
                                                    .send(UpdateStatus::Complete)
                                                    .unwrap_or_default();
                                            });
                                        }
                                    }
                                } else {
                                    tx.send(UpdateStatus::Complete).unwrap_or_default();
                                }
                            }
                            Ok(None) => {
                                show_info_dialog("You have the latest version!");
                                tx.send(UpdateStatus::Complete).unwrap_or_default();
                            }
                            Err(e) => {
                                eprintln!("Update check failed: {}", e);
                                show_info_dialog(
                                    &format!("Update check failed: {}. Please check your internet connection and try again later.", e),
                                );
                                tx.send(UpdateStatus::Error(())).unwrap_or_default();
                                let tx_clone = tx.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                    tx_clone.send(UpdateStatus::Complete).unwrap_or_default();
                                });
                            }
                        }
                    });
                }
            }

            if ui
                .button(
                    RichText::new("📦 View on GitHub")
                        .size(14.0)
                        .color(Color32::from_rgb(212, 212, 216)),
                )
                .clicked()
            {
                Self::open_url("https://github.com/kevinmartz/Rusty-Smart-Stitch");
            }
            ui.add_space(8.0);

            // Thanks to the cool people
            Self::card_frame().show(ui, |ui| {
                Self::card_header_frame().show(ui, |ui| {
                    ui.heading(
                        RichText::new("Acknowledgments")
                            .size(16.0)
                            .color(Color32::from_rgb(14, 138, 199)),
                    );
                });
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Special thanks to:")
                            .size(13.0)
                            .color(Color32::from_rgb(212, 212, 216)),
                    );

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Manas for his idea and inspiration - ")
                                .size(13.0)
                                .color(Color32::from_rgb(212, 212, 216)),
                        );
                        if ui
                            .link(
                                RichText::new("Visit GitHub")
                                    .size(13.0)
                                    .color(Color32::from_rgb(14, 138, 199)),
                            )
                            .clicked()
                        {
                            Self::open_url("https://github.com/Manas140");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("lltcggie for waifu2x-caffe - ")
                                .size(13.0)
                                .color(Color32::from_rgb(212, 212, 216)),
                        );
                        if ui
                            .link(
                                RichText::new("Visit GitHub")
                                    .size(13.0)
                                    .color(Color32::from_rgb(14, 138, 199)),
                            )
                            .clicked()
                        {
                            Self::open_url("https://github.com/lltcggie/waifu2x-caffe");
                        }
                    });
                });
            });

            // Copyright section idk why i added it but it looks cool
            ui.add_space(12.0);
            ui.label(
                RichText::new("© 2024 Rusty Smart Stitch. All rights reserved.")
                    .size(12.0)
                    .color(Color32::from_rgb(161, 161, 170)),
            );
        });
    }
}

pub fn confirm_update_dialog(release: &crate::checkupd::ReleaseInfo) -> bool {
    let message = format!(
        "A new version {} is available!\n\nRelease Notes:\n{}\n\nIf automatic update fails, you can download it manually from:\nhttps://github.com/kevinmartz/Rusty-Smart-Stitch/releases/latest",
        release.tag_name, release.body
    );

    match native_dialog::MessageDialog::new()
        .set_title("Update Available")
        .set_text(&message)
        .set_type(native_dialog::MessageType::Info)
        .show_confirm()
    {
        Ok(true) => true,
        _ => false,
    }
}

pub fn show_info_dialog(message: &str) {
    native_dialog::MessageDialog::new()
        .set_title("Information")
        .set_text(message)
        .set_type(native_dialog::MessageType::Info)
        .show_alert()
        .unwrap_or_default();
}
