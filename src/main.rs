/*
 * VeriWaveX - Professional Verilog Simulation Suite
 * Version: 1.1.0 (Linux/Windows Cross-Platform)
 * * Copyright (c) 2026 Aritrash Sarkar. All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * * Written by Aritrash Sarkar <aritrashsarkar@gmail.com>, April 2026
 */

#![windows_subsystem = "windows"]
use eframe::egui;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// --- 1. SETTINGS & PERSISTENCE ---

#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    recent_projects: Vec<PathBuf>,
}

impl AppSettings {
    fn load() -> Self {
        fs::read_to_string("settings.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write("settings.json", json);
        }
    }
}

// --- 2. CUSTOM VERILOG SYNTAX ---

fn verilog_syntax() -> Syntax {
    Syntax {
        language: "Verilog",
        case_sensitive: true,
        comment: "//",
        comment_multiline: ["/*", "*/"],
        keywords: vec![
            "module", "endmodule", "input", "output", "inout", "reg", "wire",
            "assign", "always", "initial", "begin", "end", "posedge", "negedge",
            "if", "else", "case", "endcase", "default", "parameter", "localparam", "generate", "genvar"
        ].into_iter().collect(),
        types: vec!["wire", "reg", "integer", "time", "real"].into_iter().collect(),
        special: vec!["$display", "$finish", "$dumpfile", "$dumpvars", "$time", "$stop", "$monitor"].into_iter().collect(),
        hyperlinks: BTreeSet::new(),
        // FIXED: Using char literal ' instead of string "
        quotes: BTreeSet::from(['"']), 
    }
}

// --- 3. APP DATA STRUCTURES ---

#[derive(Serialize, Deserialize, Clone, Default)]
struct Project {
    name: String,
    root: PathBuf,
    source_files: Vec<PathBuf>,
}

#[derive(PartialEq)]
enum AppState { Splash, Startup, ProjectWizard, Editor }

#[derive(PartialEq)]
enum WizardType { Module, Testbench }

struct VerilogApp {
    state: AppState,
    settings: AppSettings,
    splash_timer: f32,
    project: Option<Project>,
    
    code: String,
    active_file: Option<PathBuf>,
    console_output: String,

    show_file_wizard: bool,
    wiz_type: WizardType,
    wiz_mod_name: String,
    wiz_pins: Vec<(String, String)>,
    new_proj_name: String,
    new_proj_path: Option<PathBuf>,
}

impl Default for VerilogApp {
    fn default() -> Self {
        Self {
            state: AppState::Splash,
            settings: AppSettings::load(),
            splash_timer: 0.0,
            project: None,
            code: String::new(),
            active_file: None,
            console_output: "VeriWaveX v1.0.1 Ready.\n".to_string(),
            show_file_wizard: false,
            wiz_type: WizardType::Module,
            wiz_mod_name: "my_component".to_string(),
            wiz_pins: vec![("clk".into(), "input".into()), ("rst".into(), "input".into())],
            new_proj_name: "NewProject".to_string(),
            new_proj_path: None,
        }
    }
}

impl VerilogApp {
    fn update_title(&self, ctx: &egui::Context) {
        let proj_info = self.project.as_ref().map(|p| p.name.clone()).unwrap_or("No Project".into());
        let file_info = self.active_file.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Idle".to_string());
        
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "VeriWaveX v1.0.1 - {} - [{}]", proj_info, file_info
        )));
    }

    fn handle_shortcuts(&mut self, ui: &egui::Ui) {
        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S))) {
            self.save_current_file();
            self.console_output += "File Saved.\n";
        }
        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F5))) {
            self.run_sim();
        }
        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::N))) {
            if self.state == AppState::Editor { self.show_file_wizard = true; }
        }
    }

    fn add_to_recent(&mut self, path: PathBuf) {
        self.settings.recent_projects.retain(|p| p != &path);
        self.settings.recent_projects.insert(0, path);
        if self.settings.recent_projects.len() > 5 { self.settings.recent_projects.pop(); }
        self.settings.save();
    }

    fn get_tool_path(&self, tool_name: &str) -> PathBuf {
        if cfg!(target_os = "linux") {
            return which::which(tool_name).unwrap_or_else(|_| PathBuf::from(tool_name));
        }
        let exe_ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
        let filename = format!("{}{}", tool_name, exe_ext);
        let mut root_dir = std::env::current_exe().unwrap();
        root_dir.pop();
        if root_dir.ends_with("debug") || root_dir.ends_with("release") { root_dir.pop(); root_dir.pop(); }

        let subfolder = match tool_name {
            "iverilog" | "vvp" => "vendor/iverilog/bin",
            "gtkwave" => "vendor/iverilog/gtkwave/bin", 
            _ => "",
        };

        let full_path = root_dir.join(subfolder).join(&filename);
        full_path.canonicalize().unwrap_or(full_path)
    }

    fn finalize_project_creation(&mut self) {
        if let Some(base_path) = &self.new_proj_path {
            let proj_dir = base_path.join(&self.new_proj_name);
            let _ = fs::create_dir_all(&proj_dir);
            let proj = Project { name: self.new_proj_name.clone(), root: proj_dir.clone(), source_files: vec![] };
            let config_path = proj_dir.join(format!("{}.vwx", self.new_proj_name));
            let _ = fs::write(&config_path, serde_json::to_string_pretty(&proj).unwrap());
            self.add_to_recent(config_path);
            self.project = Some(proj);
            self.state = AppState::Editor;
        }
    }

    fn open_project_by_path(&mut self, path: PathBuf) {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(proj) = serde_json::from_str::<Project>(&data) {
                self.add_to_recent(path);
                self.project = Some(proj);
                self.state = AppState::Editor;
            }
        }
    }

    fn save_current_file(&mut self) {
        if let Some(path) = &self.active_file { let _ = fs::write(path, &self.code); }
    }

    fn run_sim(&mut self) {
        self.save_current_file();
        let Some(proj) = self.project.as_ref() else { return };

        // 1. Get tool paths (Uses your existing get_tool_path logic)
        let iverilog = self.get_tool_path("iverilog");
        let vvp = self.get_tool_path("vvp");
        let gtkwave = self.get_tool_path("gtkwave");

        // 2. Prepare source file arguments
        let mut args = vec!["-o".to_string(), "sim.vvp".to_string()];
        for f in &proj.source_files {
            if let Some(s) = f.to_str() {
                args.push(s.to_string());
            }
        }

        self.console_output += "Compiling...\n";

        // 3. Construct the Compiler Command
        let mut compile_cmd = Command::new(&iverilog);
        compile_cmd.current_dir(&proj.root).args(&args);

        // --- CRITICAL CROSS-PLATFORM LOGIC ---
        // On Windows, we need IVL_ROOT to find the modules in our 'vendor' folder.
        // On Linux (WSL), the system 'iverilog' already knows where its libraries are.
        if cfg!(target_os = "windows") {
            if let Some(p1) = iverilog.parent() {
                if let Some(p2) = p1.parent() {
                    let ivl_root = p2.join("lib/ivl");
                    compile_cmd.env("IVL_ROOT", &ivl_root);
                }
            }
        }

        let compile = compile_cmd.output();

        // 4. Handle Execution Result
        match compile {
            Ok(out) if out.status.success() => {
                // Run the VVP Simulation
                let _ = Command::new(&vvp)
                    .current_dir(&proj.root)
                    .arg("sim.vvp")
                    .output();

                // Verify VCD output and launch GTKWave
                if proj.root.join("dump.vcd").exists() {
                    let _ = Command::new(&gtkwave)
                        .current_dir(&proj.root)
                        .arg("dump.vcd")
                        .spawn();

                    self.console_output += "SUCCESS: Simulation Active.\n";
                } else {
                    self.console_output += "ERROR: Simulation finished but 'dump.vcd' is missing.\nCheck if your testbench includes $dumpfile and $dumpvars.\n";
                }
            }
            Ok(out) => {
                self.console_output = format!(
                    "SYNTAX ERROR:\n{}",
                    String::from_utf8_lossy(&out.stderr)
                )
            }
            Err(e) => {
                self.console_output = format!(
                    "SYSTEM ERROR: {}\nCheck if iverilog/gtkwave are installed and in path.",
                    e
                )
            }
        }
    }
}

// --- 5. UI IMPLEMENTATION (Fixed Traits) ---

impl eframe::App for VerilogApp {
    // FIXED: Using the 'ui' method as requested by the compiler
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update_title(ui.ctx());
        self.handle_shortcuts(ui);

        match self.state {
            AppState::Splash => {
                self.splash_timer += ui.input(|i| i.stable_dt);
                if self.splash_timer > 2.5 { self.state = AppState::Startup; }
                ui.painter().rect_filled(ui.max_rect(), 0.0, egui::Color32::BLACK);
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(egui::include_image!("../assets/splash.png")).max_size(egui::vec2(600.0, 250.0)));
                });
                ui.ctx().request_repaint();
            }

            AppState::Startup => {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Image::new(egui::include_image!("../assets/logo.png")).max_width(120.0));
                        ui.heading(egui::RichText::new("VeriWaveX v1.1.0").size(32.0).strong());
                        ui.add_space(40.0);
                        if ui.add(egui::Button::new("➕ Create New Project").min_size(egui::vec2(280.0, 45.0))).clicked() {
                            self.state = AppState::ProjectWizard;
                        }
                        ui.add_space(10.0);
                        if ui.add(egui::Button::new("📂 Open Existing Project").min_size(egui::vec2(280.0, 45.0))).clicked() {
                            if let Some(path) = rfd::FileDialog::new().add_filter("VeriWaveX", &["vwx"]).pick_file() {
                                self.open_project_by_path(path);
                            }
                        }
                        if !self.settings.recent_projects.is_empty() {
                            ui.add_space(50.0);
                            ui.label(egui::RichText::new("RECENT PROJECTS").strong());
                            let recents = self.settings.recent_projects.clone();
                            for path in recents.iter().take(3) {
                                if ui.link(format!("📁 {}", path.display())).clicked() {
                                    self.open_project_by_path(path.clone());
                                }
                            }
                        }
                    });
                });
            }

            AppState::ProjectWizard => {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Project Settings");
                    ui.add_space(20.0);
                    ui.horizontal(|ui| { ui.add_space(ui.available_width()*0.35); ui.label("Name: "); ui.text_edit_singleline(&mut self.new_proj_name); });
                    ui.add_space(10.0);
                    if ui.button("📁 Browse Location").clicked() { self.new_proj_path = rfd::FileDialog::new().pick_folder(); }
                    if let Some(p) = &self.new_proj_path {
                        ui.label(format!("Save to: {}", p.display()));
                        ui.add_space(20.0);
                        if ui.button("✅ Initialize").clicked() { self.finalize_project_creation(); }
                    }
                    ui.add_space(40.0);
                    if ui.button("⬅ Cancel").clicked() { self.state = AppState::Startup; }
                });
            }

            AppState::Editor => {
                // Status Bar
                egui::Panel::bottom("status_bar").default_size(24.0).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("READY").size(10.0).strong());
                        ui.separator();
                        if let Some(p) = &self.project { ui.label(egui::RichText::new(format!("PROJ: {}", p.name)).size(10.0)); }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(egui::RichText::new("Verilog").size(10.0)); });
                    });
                });

                // Navigator
                egui::Panel::left("nav_panel").default_size(220.0).show_inside(ui, |ui| {
                    ui.add_space(10.0);
                    ui.heading("📁 Navigator");
                    if ui.button("+ New File").clicked() { self.show_file_wizard = true; }
                    ui.separator();
                    let files = self.project.as_ref().map(|p| p.source_files.clone()).unwrap_or_default();
                    for file in files {
                        let name = file.file_name().unwrap().to_string_lossy();
                        if ui.selectable_label(self.active_file.as_ref() == Some(&file), name).clicked() {
                            self.save_current_file();
                            if let Ok(c) = fs::read_to_string(&file) { self.code = c; self.active_file = Some(file); }
                        }
                    }
                });

                // Console
                egui::Panel::bottom("log_panel").resizable(true).default_size(160.0).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Console");
                        if ui.button("🚀 Simulate").clicked() { self.run_sim(); }
                        if ui.button("🗑 Clear").clicked() { self.console_output.clear(); }
                    });
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        ui.add(egui::Label::new(&self.console_output).wrap());
                    });
                });

                // Central Editor
                if self.active_file.is_some() {
                    CodeEditor::default()
                        .id_source("main_editor")
                        .with_theme(ColorTheme::GRUVBOX)
                        .with_syntax(verilog_syntax())
                        .with_numlines(true)
                        .with_fontsize(18.0)
                        .show(ui, &mut self.code);
                } else {
                    ui.centered_and_justified(|ui| { ui.label("Select a file to begin."); });
                }
            }
        }

        // File Wizard Window (Needs to be at the end of the ui function)
        if self.show_file_wizard {
            egui::Window::new("New File").collapsible(false).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)).show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.wiz_type, WizardType::Module, "Module");
                    ui.selectable_value(&mut self.wiz_type, WizardType::Testbench, "Testbench");
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| { ui.label("Name:"); ui.text_edit_singleline(&mut self.wiz_mod_name); });

                if self.wiz_type == WizardType::Module {
                    ui.separator();
                    let mut to_rem = None;
                    for (i, (n, d)) in self.wiz_pins.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(n);
                            egui::ComboBox::from_id_salt(i).selected_text(d.clone()).show_ui(ui, |ui| {
                                ui.selectable_value(d, "input".into(), "input");
                                ui.selectable_value(d, "output".into(), "output");
                            });
                            if ui.button("❌").clicked() { to_rem = Some(i); }
                        });
                    }
                    if let Some(i) = to_rem { self.wiz_pins.remove(i); }
                    if ui.button("➕ Pin").clicked() { self.wiz_pins.push(("new_pin".into(), "input".into())); }
                }

                ui.add_space(10.0);
                if ui.button("➕ Create").clicked() {
                    let generated_code = match self.wiz_type {
                        WizardType::Module => {
                            let mut s = format!("module {} (\n", self.wiz_mod_name);
                            for (i, (n, d)) in self.wiz_pins.iter().enumerate() {
                                let c = if i == self.wiz_pins.len() - 1 { "" } else { "," };
                                s.push_str(&format!("    {} {}{}\n", d, n, c));
                            }
                            s.push_str(");\n\n// Logic\n\nendmodule"); s
                        },
                        WizardType::Testbench => {
                            format!("module {}_tb;\nreg clk;\n\n{} uut (.clk(clk));\n\ninitial begin\n    $dumpfile(\"dump.vcd\");\n    $dumpvars(0, {}_tb);\n    clk = 0; #100 $finish;\nend\n\nalways #5 clk = ~clk;\nendmodule", 
                            self.wiz_mod_name, self.wiz_mod_name, self.wiz_mod_name)
                        }
                    };
                    if let Some(proj) = &mut self.project {
                        let fname = if self.wiz_type == WizardType::Testbench { format!("{}_tb.v", self.wiz_mod_name) } else { format!("{}.v", self.wiz_mod_name) };
                        let path = proj.root.join(fname);
                        let _ = fs::write(&path, &generated_code);
                        proj.source_files.push(path.clone());
                        self.active_file = Some(path);
                        self.code = generated_code;
                        let _ = fs::write(proj.root.join(format!("{}.vwx", proj.name)), serde_json::to_string_pretty(proj).unwrap());
                    }
                    self.show_file_wizard = false;
                }
            });
        }
    }
}

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/logo.png");
    let icon = image::load_from_memory(icon_bytes).ok().map(|img| {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData { rgba: rgba.into_raw(), width, height }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_inner_size([1200.0, 800.0])
            .with_icon(icon.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native("VeriWaveX v1.1.0", options, Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(VerilogApp::default()))
    }))
}
