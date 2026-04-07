pub mod menu;
pub mod panels;
pub mod wizards;
pub mod about;
pub mod schematic;

use eframe::egui;
use egui_code_editor::{CodeEditor, ColorTheme};
use crate::app::VerilogApp;
use crate::models::AppState;
use crate::logic;

// Re-export functions for cleaner access in app.rs
pub use menu::draw_menu_bar;
pub use panels::{draw_status_bar, draw_navigator, draw_console};
pub use wizards::draw_file_wizard;
pub use about::draw_about_window;
pub use schematic::draw_schematic_window;

pub fn draw_splash(app: &mut VerilogApp, ui: &mut egui::Ui) {
    app.splash_timer += ui.input(|i| i.stable_dt);
    if app.splash_timer > 2.5 { app.state = AppState::Startup; }
    
    ui.painter().rect_filled(ui.max_rect(), 0.0, egui::Color32::BLACK);
    ui.centered_and_justified(|ui| {
        // NOTE: Two dots now because we are inside src/ui/
        ui.add(egui::Image::new(egui::include_image!("../../assets/splash.png"))
            .max_size(egui::vec2(600.0, 250.0)));
    });
    ui.ctx().request_repaint();
}

pub fn draw_startup(app: &mut VerilogApp, ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add(egui::Image::new(egui::include_image!("../../assets/logo.png")).max_width(120.0));
            ui.heading(egui::RichText::new("VeriWaveX v2.0.0").size(32.0).strong());
            ui.add_space(40.0);
            
            if ui.add(egui::Button::new("➕ Create New Project").min_size(egui::vec2(280.0, 45.0))).clicked() {
                app.state = AppState::ProjectWizard;
            }
            ui.add_space(10.0);
            if ui.add(egui::Button::new("📂 Open Existing Project").min_size(egui::vec2(280.0, 45.0))).clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("VeriWaveX", &["vwx"]).pick_file() {
                    app.open_project_by_path(path);
                }
            }

            if !app.settings.recent_projects.is_empty() {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("RECENT PROJECTS").strong());
                let recents = app.settings.recent_projects.clone();
                for path in recents.iter().take(3) {
                    if ui.link(format!("📁 {}", path.display())).clicked() {
                        app.open_project_by_path(path.clone());
                    }
                }
            }
        });
    });
}

pub fn draw_project_wizard(app: &mut VerilogApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(100.0);
        ui.heading("Project Settings");
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() * 0.35);
            ui.label("Name: ");
            ui.text_edit_singleline(&mut app.new_proj_name);
        });
        ui.add_space(10.0);
        if ui.button("📁 Browse Location").clicked() {
            app.new_proj_path = rfd::FileDialog::new().pick_folder();
        }
        if let Some(p) = &app.new_proj_path {
            ui.label(format!("Save to: {}", p.display()));
            ui.add_space(20.0);
            if ui.button("✅ Initialize").clicked() {
                app.finalize_project_creation();
            }
        }
        ui.add_space(40.0);
        if ui.button("⬅ Cancel").clicked() { app.state = AppState::Startup; }
    });
}

pub fn draw_editor(app: &mut VerilogApp, ui: &mut egui::Ui) {
    if app.active_file.is_some() {
        CodeEditor::default()
            .id_source("main_editor")
            .with_theme(if ui.ctx().global_style().visuals.dark_mode { 
                ColorTheme::GRUVBOX 
            } else { 
                ColorTheme::GITHUB_LIGHT 
            })
            .with_syntax(logic::verilog_syntax())
            .with_numlines(true)
            .with_fontsize(18.0)
            .show(ui, &mut app.code);
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a file from the Navigator to begin coding.");
        });
    }
}