use eframe::egui;
use std::fs;
use crate::app::VerilogApp;

pub fn draw_status_bar(app: &mut VerilogApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("status_bar")
        .default_size(24.0)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("READY").size(10.0).strong());
                ui.separator();
                if let Some(p) = &app.project {
                    ui.label(egui::RichText::new(format!("PROJ: {}", p.name)).size(10.0));
                }
            });
        });
}

pub fn draw_navigator(app: &mut VerilogApp, ui: &mut egui::Ui) {
    egui::Panel::left("nav_panel")
        .default_size(220.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.add_space(10.0);
            ui.heading("📁 Navigator");
            if ui.button("+ New File").clicked() { app.show_file_wizard = true; }
            ui.separator();
            
            let files = app.project.as_ref().map(|p| p.source_files.clone()).unwrap_or_default();
            for file in files {
                let name = file.file_name().unwrap_or_default().to_string_lossy();
                if ui.selectable_label(app.active_file.as_ref() == Some(&file), name).clicked() {
                    app.save_current_file();
                    if let Ok(content) = fs::read_to_string(&file) {
                        app.code = content;
                        app.active_file = Some(file);
                    }
                }
            }
        });
}

pub fn draw_console(app: &mut VerilogApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("log_panel")
        .resizable(true)
        .default_size(160.0)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Console");
                if ui.button("🗑 Clear").clicked() { app.console_output.clear(); }
            });
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.add(egui::Label::new(&app.console_output).wrap());
            });
        });
}