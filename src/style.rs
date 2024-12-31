use eframe::egui::{self, Color32, Margin, Rounding, Stroke, Vec2, CentralPanel, RichText};
use crate::{RustySmartStitchApp, Tab};

pub fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    style.visuals.window_fill = Color32::from_rgb(35, 35, 35);       // Darker background
    style.visuals.panel_fill = Color32::from_rgb(35, 35, 35);        // Darker background
    style.visuals.faint_bg_color = Color32::from_rgb(28, 28, 28);    // same as above
    style.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 45);      // Button background
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(70, 70, 70);        // Pressed button
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 55, 55);       // Hover state
    style.visuals.selection.bg_fill = Color32::from_rgb(70, 70, 70);            // Selected items

    // Completely sharp edges
    style.visuals.window_rounding = Rounding::same(0.0);
    style.visuals.menu_rounding = Rounding::same(0.0);
    style.visuals.widgets.noninteractive.rounding = Rounding::same(0.0);
    style.visuals.widgets.inactive.rounding = Rounding::same(0.0);
    style.visuals.widgets.hovered.rounding = Rounding::same(0.0);
    style.visuals.widgets.active.rounding = Rounding::same(0.0);

    // Crisp, pixel-ish borders
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(100, 100, 100));
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(80, 80, 80));
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 60));
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;

    // Text colors (using fg_stroke instead of text_color because i dont know how to do it the right way)
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 200, 200));
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 220, 220));

    // Spacing for more blocky feel i think?
    style.spacing.item_spacing = Vec2::new(6.0, 6.0);
    style.spacing.window_margin = Margin::same(6.0);
    style.spacing.button_padding = Vec2::new(6.0, 4.0);

    ctx.set_style(style);
} 

impl eframe::App for RustySmartStitchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        setup_style(ctx);

        if self.processing {
            self.update_progress(ctx);
        }

        // Check for update status
        if let Some(rx) = &self.update_status_rx {
            match rx.try_recv() {
                Ok(status) => {
                    self.current_update_status = Some(status);
                    ctx.request_repaint();
                }
                Err(_) => {}
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            // Disable all interactions when processing
            let enabled = !self.processing;
            
            egui::TopBottomPanel::top("tabs").show_inside(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.current_tab, Tab::Main, "Main");
                        
                        // Advanced tab indicator
                        let advanced_text = if self.has_active_advanced_settings() {
                            RichText::new("Advanced •")
                                .color(Color32::from_rgb(69, 133, 136))
                                .strong()
                        } else {
                            RichText::new("Advanced")
                        };
                        ui.selectable_value(&mut self.current_tab, Tab::Advanced, advanced_text);
                        
                        // Waifu2x tab indicator
                        let waifu2x_text = if self.has_active_waifu2x() {
                            RichText::new("Waifu2x •")
                                .color(Color32::from_rgb(69, 133, 136))
                                .strong()
                        } else {
                            RichText::new("Waifu2x")
                        };
                        ui.selectable_value(&mut self.current_tab, Tab::Waifu2x, waifu2x_text);
                        
                        ui.selectable_value(&mut self.current_tab, Tab::About, "About");
                    });
                });
            });

            match self.current_tab {
                Tab::Main => self.show_main(ui, enabled, ctx),
                Tab::About => self.show_about(ui),
                Tab::Advanced => {
                    ui.add_enabled_ui(enabled, |ui| {
                        self.show_advanced(ui)
                    });
                },
                Tab::Waifu2x => {
                    ui.add_enabled_ui(enabled, |ui| {
                        self.show_waifu2x(ui)
                    });
                }
            }
        });
    }
}