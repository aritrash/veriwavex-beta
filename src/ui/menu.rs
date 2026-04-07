/*
 * VeriWaveX - Menu Bar Implementation
 * File: src/ui/menu.rs
 */

use eframe::egui;
use crate::app::VerilogApp;
use crate::models::{AppState, AppTheme};
use crate::logic;

pub fn draw_menu_bar(app: &mut VerilogApp, ui: &mut egui::Ui) {
    // FIXED: Using the explicit menu module and adding type annotations
    egui::menu::bar(ui, |ui: &mut egui::Ui| {
        
        // --- 1. FILE MENU ---
        ui.menu_button("File", |ui: &mut egui::Ui| {
            if ui.button("➕ New Project").clicked() {
                app.state = AppState::ProjectWizard;
                ui.close_menu();
            }
            if ui.button("📂 Open Project").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("VeriWaveX", &["vwx"])
                    .pick_file() 
                {
                    app.open_project_by_path(path);
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("💾 Save (Ctrl+S)").clicked() {
                app.save_current_file();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("📥 Import Xilinx ISE").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Xilinx ISE", &["xise"])
                    .pick_file() 
                {
                    match logic::import_xilinx_ise(path) {
                        Ok(proj) => {
                            app.project = Some(proj);
                            app.state = AppState::Editor;
                            app.console_output += "SUCCESS: ISE Project Imported.\n";
                        }
                        Err(e) => app.console_output += &format!("IMPORT ERROR: {}\n", e),
                    }
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Exit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        // --- 2. VIEW MENU ---
        ui.menu_button("View", |ui: &mut egui::Ui| {
            ui.label("Theme");
            ui.radio_value(&mut app.theme, AppTheme::System, "🖥 System Default");
            ui.radio_value(&mut app.theme, AppTheme::Light, "☀ Light Mode");
            ui.radio_value(&mut app.theme, AppTheme::Dark, "🌙 Dark Mode");
        });

        // --- 3. TOOLS MENU ---
        ui.menu_button("Tools", |ui: &mut egui::Ui| {
            if ui.button("📊 Schematic Helper").clicked() {
                app.show_schematic = true;
                ui.close_menu();
            }
        });

        // --- 4. SIMULATE MENU ---
        ui.menu_button("Simulate", |ui: &mut egui::Ui| {
            if ui.button("🚀 Run (F5)").clicked() {
                app.run_sim();
                ui.close_menu();
            }
        });

        // --- 5. HELP MENU ---
        ui.menu_button("Help", |ui: &mut egui::Ui| {
            if ui.button("ℹ About VeriWaveX").clicked() {
                app.show_about = true;
                ui.close_menu();
            }
        });
    });
}