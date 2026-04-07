/*
 * VeriWaveX - About Window
 * File: src/ui/about.rs
 */

use eframe::egui;

pub fn draw_about_window(ctx: &egui::Context, open: &mut bool) {
    // FIXED: Removed .open() to avoid the double borrow conflict
    // The window is already conditionally rendered in app.rs
    egui::Window::new("About VeriWaveX")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(egui::include_image!("../../assets/logo.png")).max_width(80.0));
                ui.add_space(5.0);
                ui.heading("VeriWaveX v2.0.0");
                ui.label("Professional Verilog IDE & Simulation Suite");
                ui.separator();
                
                ui.label(egui::RichText::new("Developed by Aritrash Sarkar").strong());
                ui.label("Innovation Ambassador | MSIT CSE");
                ui.add_space(10.0);
                
                ui.label("© 2026 All Rights Reserved.");
                ui.hyperlink_to("GitHub Repository", "https://github.com/aritrashsarkar/veriwavex");
                
                ui.add_space(15.0);
                if ui.button("Close").clicked() {
                    *open = false; 
                }
            });
        });
}