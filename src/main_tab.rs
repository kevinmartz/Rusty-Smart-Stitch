use eframe::egui::{self, Align, Color32, Layout, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use egui::RadioButton;
use egui_extras::TableBuilder;
use std::path::PathBuf;

use crate::RustySmartStitchApp;

const PADDING: f32 = 32.0; // Space around stuff
const SPACING: f32 = 10.0; // Space between things
const BUTTON_HEIGHT: f32 = 35.0; // Buttons thicc enough to click
const DRAG_AREA_HEIGHT: f32 = 200.0; // Big enough to drop files in
const HEADING_SIZE: f32 = 20.0; // Main title
const SUBHEADING_SIZE: f32 = 18.0; // Section titles
const NORMAL_TEXT_SIZE: f32 = 16.0; // Regular text
const SMALL_TEXT_SIZE: f32 = 12.0; // Tiny text
const TABLE_HEIGHT: f32 = 130.0; // File list height

const SUPPORTED_FORMATS: [(&str, &str); 4] = [
    ("jpg", "JPG"),
    ("png", "PNG"),
    ("webp", "WebP"),
    ("bmp", "BMP"),
];

// File types it can process
const SUPPORTED_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "bmp", "psd"];

// Load an image from bytes so we dont have to redner it everytime
// Used for the drag n drop icon
fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

impl RustySmartStitchApp {
    pub fn show_main(&mut self, ui: &mut Ui, enabled: bool, ctx: &egui::Context) {
        self.initialize_drag_icon(ctx);
        self.show_header(ui);

        let panel_width = ui.available_width() - PADDING;

        ui.vertical(|ui| {
            ui.add_enabled_ui(enabled, |ui| {
                self.show_drag_drop_area(ui, panel_width, ctx);
            });

            ui.add_space(SPACING);
            self.show_file_controls(ui, panel_width, enabled);
            ui.add_space(SPACING * 1.5);
            self.show_parameters_section(ui, panel_width, enabled);
            ui.add_space(SPACING * 1.5);
            self.show_process_button_and_status(ui, enabled);
        });
    }

    // Load the drag n drop icon - only do it once and thanks to that guy in github for the stolen code hehe
    fn initialize_drag_icon(&mut self, ctx: &egui::Context) {
        if self.drag_icon.is_none() {
            let icon_data = include_bytes!("../assets/drag.ico");
            if let Ok(image) = load_image_from_memory(icon_data) {
                self.drag_icon =
                    Some(ctx.load_texture("drag-icon", image, egui::TextureOptions::default()));
            }
        }
    }

    // Show the app title - keep it simple if you dont like it remove the .family(egui::FontFamily::Name("PixelFont".into())) or change the font from the main.rs
    fn show_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            ui.heading(
                RichText::new("Rusty Smart Stitch".to_uppercase())
                    .size(HEADING_SIZE)
                    .color(Color32::from_rgb(217, 217, 217))
                    .family(egui::FontFamily::Name("PixelFont".into())),
            );
            ui.add_space(5.0);
        });
    }

    // The big area for drop files
    // Shows progress when processing check the proccess_handler.rs for the actual processing and the progress bar dont change anything here
    fn show_drag_drop_area(&mut self, ui: &mut Ui, panel_width: f32, ctx: &egui::Context) {
        ui.with_layout(
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                ui.set_width(panel_width);
                ui.group(|ui| {
                    ui.set_min_height(DRAG_AREA_HEIGHT);
                    ui.set_max_height(DRAG_AREA_HEIGHT);

                    let rect = ui.max_rect();
                    let base_fill = Color32::from_gray(20); // Dark background
                    let mut stroke = Stroke::new(1.0, Color32::from_gray(100)); // Gray border

                    if self.processing || !self.success_message.is_empty() {
                        self.draw_progress_area(ui, rect, DRAG_AREA_HEIGHT);
                    } else {
                        self.handle_drag_drop_ui(ui, ctx, &rect, base_fill, &mut stroke);
                    }
                });
            },
        );
    }

    // Handle the drag n drop UI state
    // Changes color when dragging files over
    fn handle_drag_drop_ui(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        rect: &egui::Rect,
        base_fill: Color32,
        stroke: &mut Stroke,
    ) {
        let is_hovering = ctx.input(|i| i.raw.hovered_files.len()) > 0;
        self.drag_hovering = is_hovering;

        if self.drag_hovering {
            *stroke = Stroke::new(2.0, Color32::from_rgb(69, 133, 136)); // Highlight when hovering
            ctx.request_repaint();
        }

        ui.painter().rect(*rect, Rounding::ZERO, base_fill, *stroke);
        self.handle_file_drops(ctx);
        self.show_drag_drop_content(ui);
    }

    fn show_drag_drop_content(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(7.0);

            if self.input_paths.is_empty() && self.pending_subfolders.is_none() {
                self.show_empty_drag_area(ui);
            } else {
                self.show_file_list(ui);
            }
        });
    }

    fn show_empty_drag_area(&self, ui: &mut Ui) {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.vertical_centered(|ui| {
                    let icon_tint = if self.drag_hovering {
                        Color32::from_rgb(100, 200, 255)
                    } else {
                        Color32::WHITE
                    };

                    if let Some(icon) = &self.drag_icon {
                        let size = egui::Vec2::new(74.0, 74.0);
                        ui.add_space(20.0);
                        ui.add(
                            egui::Image::new(icon)
                                .fit_to_exact_size(size)
                                .tint(icon_tint),
                        );
                        ui.add_space(10.0);
                    }

                    let text_color = if self.drag_hovering {
                        Color32::from_rgb(100, 200, 255)
                    } else {
                        Color32::GRAY
                    };

                    let text = if self.drag_hovering {
                        "Drop images here!"
                    } else {
                        "Drag and drop images here"
                    };

                    ui.label(
                        RichText::new(text)
                            .color(text_color)
                            .size(SMALL_TEXT_SIZE)
                            .family(egui::FontFamily::Name("PixelFont".into())),
                    );
                });
            },
        );
    }

    fn show_file_list(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.set_max_height(DRAG_AREA_HEIGHT - 20.0);

            ui.horizontal(|ui| {
                ui.add_space(10.0);

                ui.label(
                    RichText::new(format!(
                        "📄 Current folder files: {}",
                        self.input_paths.len()
                    ))
                    .color(Color32::from_rgb(100, 200, 255)),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(10.0);
                    if let Some(ref subfolders) = self.pending_subfolders {
                        if !subfolders.is_empty() {
                            ui.label(
                                RichText::new(format!(
                                    "📁 Pending subfolders: {}",
                                    subfolders.len()
                                ))
                                .color(Color32::from_rgb(255, 180, 100)),
                            );
                        }
                    }
                });
            });

            ui.add_space(5.0);

            let table_width = ui.available_width() - 24.0;
            ui.add_space(8.0);
            ui.scope(|ui| {
                ui.set_width(table_width);
                self.build_file_table(ui);
            });
        });
    }

    fn build_file_table(&self, ui: &mut Ui) {
        TableBuilder::new(ui)
            .striped(true)
            .vscroll(true)
            .max_scroll_height(TABLE_HEIGHT)
            .min_scrolled_height(0.0)
            .cell_layout(egui::Layout::left_to_right(Align::Center))
            .column(egui_extras::Column::initial(20.0).clip(true))
            .column(egui_extras::Column::remainder().clip(true))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label("•");
                    });
                });
                header.col(|ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(RichText::new("Name").color(Color32::from_rgb(180, 180, 180)));
                    });
                });
            })
            .body(|mut body| {
                for path in &self.input_paths {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.label(
                                    RichText::new("•").color(Color32::from_rgb(100, 200, 255)),
                                );
                            });
                        });
                        row.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                let file_name = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let label =
                                    ui.add(egui::Label::new(file_name).sense(Sense::click()));
                                if label.hovered() {
                                    ui.painter().rect_filled(
                                        label.rect,
                                        0.0,
                                        Color32::from_rgb(0, 60, 120),
                                    );
                                }
                            });
                        });
                    });
                }

                if let Some(ref subfolders) = self.pending_subfolders {
                    if !subfolders.is_empty() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    ui.label("⌛");
                                });
                            });
                            row.col(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new("Pending Subfolders")
                                            .italics()
                                            .color(Color32::from_rgb(180, 180, 180)),
                                    );
                                });
                            });
                        });

                        let subfolder_vec: Vec<_> = subfolders.iter().collect();
                        for subfolder in subfolder_vec {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new("📁")
                                                .color(Color32::from_rgb(255, 180, 100)),
                                        );
                                    });
                                });
                                row.col(|ui| {
                                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                        let folder_name = subfolder
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .to_string();
                                        let label = ui.add(
                                            egui::Label::new(folder_name).sense(Sense::click()),
                                        );
                                        if label.hovered() {
                                            ui.painter().rect_filled(
                                                label.rect,
                                                0.0,
                                                Color32::from_rgb(0, 60, 120),
                                            );
                                        }
                                    });
                                });
                            });
                        }
                    }
                }
            });
    }

    fn show_file_controls(&mut self, ui: &mut Ui, panel_width: f32, enabled: bool) {
        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
            ui.set_width(panel_width);
            ui.group(|ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    self.show_file_buttons(ui, panel_width);
                    self.show_output_directory_input(ui, panel_width);
                });
            });
        });
    }

    fn show_file_buttons(&mut self, ui: &mut Ui, panel_width: f32) {
        ui.horizontal(|ui| {
            let button_size = Vec2::new((panel_width - 48.0) / 4.0, BUTTON_HEIGHT);

            ui.horizontal(|ui| {
                self.show_select_files_button(ui, button_size);
                ui.add_space(5.0);
                self.show_select_folder_button(ui, button_size);
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                self.show_clear_button(ui, button_size);
                ui.add_space(5.0);
                self.show_output_dir_button(ui, button_size);
            });
        });
    }

    fn show_select_files_button(&mut self, ui: &mut Ui, button_size: Vec2) {
        let select_files_button = egui::Button::new("📂 Select Files").min_size(button_size);
        if ui
            .add_enabled(self.input_paths.is_empty(), select_files_button)
            .on_hover_text(if self.input_paths.is_empty() {
                "Select image files to process"
            } else {
                "Clear existing files first"
            })
            .clicked()
        {
            self.handle_select_files(&SUPPORTED_EXTENSIONS);
        }
    }

    fn show_select_folder_button(&mut self, ui: &mut Ui, button_size: Vec2) {
        let select_folder_button = egui::Button::new("📁 Select Folder").min_size(button_size);
        if ui
            .add_enabled(self.input_paths.is_empty(), select_folder_button)
            .on_hover_text(if self.input_paths.is_empty() {
                "Select a folder with images (includes subfolders)"
            } else {
                "Clear existing files first"
            })
            .clicked()
        {
            self.handle_select_folder();
        }
    }

    fn show_clear_button(&mut self, ui: &mut Ui, button_size: Vec2) {
        if ui
            .add_sized(button_size, egui::Button::new("🗑️ Clear"))
            .on_hover_text("Clear all selections")
            .clicked()
        {
            self.clear_all();
        }
    }

    fn show_output_dir_button(&mut self, ui: &mut Ui, button_size: Vec2) {
        if ui
            .add_sized(button_size, egui::Button::new("📁 Output Dir"))
            .on_hover_text("Select output directory")
            .clicked()
        {
            self.handle_output_dir_selection();
        }
    }

    // Text box showing where we're saving to, if you want to show it all the time remove the if self.output_dir.is_some()
    fn show_output_directory_input(&mut self, ui: &mut Ui, panel_width: f32) {
        if self.output_dir.is_some() {
            ui.add_space(5.0);
            let response = ui.add_sized(
                Vec2::new(panel_width - 20.0, 30.0),
                egui::TextEdit::singleline(&mut self.manual_output_dir)
                    .font(egui::TextStyle::Monospace)
                    .vertical_align(egui::Align::Center),
            );
            if response.changed() {
                self.output_dir = Some(PathBuf::from(&self.manual_output_dir));
            }
        }
    }

    fn show_parameters_section(&mut self, ui: &mut Ui, panel_width: f32, enabled: bool) {
        ui.add_enabled_ui(enabled, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.set_width(panel_width);
                ui.group(|ui| {
                    ui.heading(
                        RichText::new("Parameters")
                            .size(SUBHEADING_SIZE)
                            .color(Color32::from_rgb(217, 217, 217)),
                    );
                    ui.add_space(5.0);
                    self.show_parameters(ui);
                });
            });
        });
    }

    pub fn show_parameters(&mut self, ui: &mut Ui) {
        self.show_numeric_parameters(ui);
        ui.add_space(SPACING);
        self.show_format_selection(ui);
    }

    fn show_numeric_parameters(&mut self, ui: &mut Ui) {
        let mut height = self.rough_output_height.clone();
        let mut sensitivity = self.sensitivity.clone();
        let mut scan_step = self.scan_step.clone();
        let mut quality = self.output_quality.clone();

        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                ui.set_width(90.0);
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.label(RichText::new("Height").size(NORMAL_TEXT_SIZE));
                });
                let response = ui.add_sized(
                    Vec2::new(90.0, 30.0),
                    egui::TextEdit::singleline(&mut height)
                        .hint_text("100-20000")
                        .font(egui::TextStyle::Monospace)
                        .horizontal_align(egui::Align::Center)
                        .vertical_align(egui::Align::Center),
                );
                if response.changed() {
                    height = height.chars().filter(|c| c.is_digit(10)).collect();
                }
            });

            ui.add_space(SPACING);

            ui.vertical(|ui| {
                ui.set_width(90.0);
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.label(RichText::new("Sensitivity").size(NORMAL_TEXT_SIZE));
                });
                let response = ui.add_sized(
                    Vec2::new(90.0, 30.0),
                    egui::TextEdit::singleline(&mut sensitivity)
                        .hint_text("1-100")
                        .font(egui::TextStyle::Monospace)
                        .horizontal_align(egui::Align::Center)
                        .vertical_align(egui::Align::Center),
                );
                if response.changed() {
                    sensitivity = sensitivity.chars().filter(|c| c.is_digit(10)).collect();
                }
            });

            ui.add_space(SPACING);

            ui.vertical(|ui| {
                ui.set_width(90.0);
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.label(RichText::new("Scan Step").size(NORMAL_TEXT_SIZE));
                });
                let response = ui.add_sized(
                    Vec2::new(90.0, 30.0),
                    egui::TextEdit::singleline(&mut scan_step)
                        .hint_text("1-20")
                        .font(egui::TextStyle::Monospace)
                        .horizontal_align(egui::Align::Center)
                        .vertical_align(egui::Align::Center),
                );
                if response.changed() {
                    scan_step = scan_step.chars().filter(|c| c.is_digit(10)).collect();
                }
            });

            ui.add_space(SPACING);

            ui.vertical(|ui| {
                ui.set_width(90.0);
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    let quality_label =
                        if self.output_format == "jpg" || self.output_format == "webp" {
                            RichText::new("Quality").size(NORMAL_TEXT_SIZE)
                        } else {
                            RichText::new("Quality")
                                .size(NORMAL_TEXT_SIZE)
                                .color(Color32::from_gray(100))
                        };
                    ui.label(quality_label);
                });
                let response = ui
                    .add_enabled_ui(
                        self.output_format == "jpg" || self.output_format == "webp",
                        |ui| {
                            ui.add_sized(
                                Vec2::new(90.0, 30.0),
                                egui::TextEdit::singleline(&mut quality)
                                    .hint_text("1-100")
                                    .font(egui::TextStyle::Monospace)
                                    .horizontal_align(egui::Align::Center)
                                    .vertical_align(egui::Align::Center),
                            )
                        },
                    )
                    .inner;
                if response.changed() {
                    quality = quality.chars().filter(|c| c.is_digit(10)).collect();
                }
            });
        });

        self.rough_output_height = height;
        self.sensitivity = sensitivity;
        self.scan_step = scan_step;
        self.output_quality = quality;
    }

    fn show_format_selection(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Format")
                        .size(NORMAL_TEXT_SIZE)
                        .color(Color32::from_rgb(217, 217, 217)),
                );
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 15.0;
                    for (value, label) in SUPPORTED_FORMATS {
                        if ui
                            .add_sized(
                                Vec2::new(70.0, 25.0),
                                RadioButton::new(
                                    self.output_format == value,
                                    RichText::new(label).size(NORMAL_TEXT_SIZE),
                                ),
                            )
                            .clicked()
                        {
                            self.update_format(value);
                        }
                    }
                });
            });
        });
    }

    fn update_format(&mut self, new_format: &str) {
        self.output_format = new_format.to_string();
        self.setup_output_directory();
    }

    fn show_process_button_and_status(&mut self, ui: &mut Ui, enabled: bool) {
        ui.vertical_centered(|ui| {
            let button = ui.add_enabled(
                enabled
                    && !self.input_paths.is_empty()
                    && self.output_dir.is_some()
                    && !self.processing,
                egui::Button::new(RichText::new("Process Images").size(SUBHEADING_SIZE).color(
                    if self.processing {
                        Color32::GRAY
                    } else {
                        Color32::WHITE
                    },
                ))
                .min_size(Vec2::new(200.0, 40.0)),
            );

            if button.clicked() {
                self.start_processing();
            }

            ui.add_space(SPACING * 1.5);
        });
    }
}
