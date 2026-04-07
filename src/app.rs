/*
 * VeriWaveX - Professional Verilog Simulation Suite
 * File: src/app.rs
 * Version: 2.0.0 (Linux/Windows Cross-Platform)
 */

use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::models::*;
use crate::logic;
use crate::ui;

pub struct VerilogApp {
    pub state: AppState,
    pub settings: UserSettings,
    pub theme: AppTheme,
    pub splash_timer: f32,
    pub project: Option<Project>,
    
    pub code: String,
    pub active_file: Option<PathBuf>,
    pub console_output: String,

    pub show_file_wizard: bool,
    pub wiz_type: WizardType,
    pub wiz_mod_name: String,
    pub wiz_pins: Vec<(String, String)>,
    pub new_proj_name: String,
    pub new_proj_path: Option<PathBuf>,
    pub show_about: bool,
    pub show_schematic: bool,
    pub schematic_image: Option<egui::TextureHandle>,
}

impl Default for VerilogApp {
    fn default() -> Self {
        Self {
            state: AppState::Splash,
            settings: logic::load_settings(),
            theme: AppTheme::System,
            splash_timer: 0.0,
            project: None,
            code: String::new(),
            active_file: None,
            console_output: "VeriWaveX v2.0.0 Ready.\n".to_string(),
            show_file_wizard: false,
            wiz_type: WizardType::Module,
            wiz_mod_name: "my_component".to_string(),
            wiz_pins: vec![("clk".into(), "input".into()), ("rst".into(), "input".into())],
            new_proj_name: "NewProject".to_string(),
            new_proj_path: None,
            show_about: false,
            show_schematic: false,
            schematic_image: None,
        }
    }
}

impl VerilogApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn update_title(&self, ctx: &egui::Context) {
        let proj_info = self.project.as_ref().map(|p| p.name.clone()).unwrap_or("No Project".into());
        let file_info = self.active_file.as_ref()
            .and_then(|p: &PathBuf| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Idle".to_string());
        
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "VeriWaveX v2.0.0 - {} - [{}]", proj_info, file_info
        )));
    }

    pub fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S))) {
            self.save_current_file();
            self.console_output += "File Saved.\n";
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F5))) {
            self.run_sim();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::N))) {
            if self.state == AppState::Editor { self.show_file_wizard = true; }
        }
    }

    pub fn open_project_by_path(&mut self, path: PathBuf) {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(proj) = serde_json::from_str::<Project>(&data) {
                logic::add_to_recent(&mut self.settings, path);
                self.project = Some(proj);
                self.state = AppState::Editor;
            }
        }
    }

    pub fn finalize_project_creation(&mut self) {
        if let Some(base_path) = &self.new_proj_path {
            let proj_dir = base_path.join(&self.new_proj_name);
            let _ = fs::create_dir_all(&proj_dir);
            let proj = Project { name: self.new_proj_name.clone(), root: proj_dir.clone(), source_files: vec![] };
            let config_path = proj_dir.join(format!("{}.vwx", self.new_proj_name));
            let _ = fs::write(&config_path, serde_json::to_string_pretty(&proj).unwrap());
            logic::add_to_recent(&mut self.settings, config_path);
            self.project = Some(proj);
            self.state = AppState::Editor;
        }
    }

    pub fn save_current_file(&mut self) {
        if let Some(path) = &self.active_file { 
            let _ = fs::write(path, &self.code); 
        }
    }

    pub fn run_sim(&mut self) {
        self.save_current_file();
        let Some(proj) = self.project.as_ref() else { return };

        let iverilog = logic::get_tool_path("iverilog");
        let vvp = logic::get_tool_path("vvp");
        let gtkwave = logic::get_tool_path("gtkwave");

        let mut args = vec!["-o".to_string(), "sim.vvp".to_string()];
        for f in &proj.source_files {
            if let Some(s) = f.to_str() { args.push(s.to_string()); }
        }

        self.console_output += "Compiling...\n";
        let mut compile_cmd = Command::new(&iverilog);
        compile_cmd.current_dir(&proj.root).args(&args);

        if cfg!(target_os = "windows") {
            if let Some(p2) = iverilog.parent().and_then(|p| p.parent()) {
                compile_cmd.env("IVL_ROOT", p2.join("lib/ivl"));
            }
        }

        match compile_cmd.output() {
            Ok(out) if out.status.success() => {
                let _ = Command::new(&vvp).current_dir(&proj.root).arg("sim.vvp").output();
                if proj.root.join("dump.vcd").exists() {
                    let _ = Command::new(&gtkwave).current_dir(&proj.root).arg("dump.vcd").spawn();
                    self.console_output += "SUCCESS: Simulation Active.\n";
                } else {
                    self.console_output += "ERROR: 'dump.vcd' missing.\n";
                }
            }
            Ok(out) => self.console_output = format!("SYNTAX ERROR:\n{}", String::from_utf8_lossy(&out.stderr)),
            Err(e) => self.console_output = format!("SYSTEM ERROR: {}\n", e),
        }
    }
}

impl eframe::App for VerilogApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 1. Theme (Fetch ctx inline to avoid long-lived borrow)
        match self.theme {
            AppTheme::Dark => ui.ctx().set_visuals(egui::Visuals::dark()),
            AppTheme::Light => ui.ctx().set_visuals(egui::Visuals::light()),
            _ => {}
        }

        self.update_title(ui.ctx());
        self.handle_shortcuts(ui.ctx());

        // 2. Layout
        if self.state != AppState::Splash {
            egui::Panel::top("top_menu").show_inside(ui, |ui| {
                ui::draw_menu_bar(self, ui);
            });

            if self.state == AppState::Editor {
                // FIXED: Passing 'ui' instead of 'ctx' to panels 
                // and calling ui.ctx() inside them if needed.
                ui::draw_status_bar(self, ui);
                ui::draw_console(self, ui);
                ui::draw_navigator(self, ui);
            }
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            match self.state {
                AppState::Splash => ui::draw_splash(self, ui),
                AppState::Startup => ui::draw_startup(self, ui),
                AppState::ProjectWizard => ui::draw_project_wizard(self, ui),
                AppState::Editor => ui::draw_editor(self, ui),
            }
        });

        // 3. Modals (Global context is safe here because CentralPanel borrow has ended)
        let ctx = ui.ctx();
        if self.show_file_wizard { ui::draw_file_wizard(self, ctx); }
        if self.show_about { ui::draw_about_window(ctx, &mut self.show_about); }
        if self.show_schematic { ui::draw_schematic_window(self, ctx); }
    }
}