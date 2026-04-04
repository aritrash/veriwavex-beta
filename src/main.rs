use eframe::egui;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// --- Data Structures ---

#[derive(Serialize, Deserialize, Clone, Default)]
struct Project {
    name: String,
    root: PathBuf,
    source_files: Vec<PathBuf>,
}

#[derive(PartialEq)]
enum AppState {
    Splash,
    Startup,
    ProjectWizard,
    Editor,
}

#[derive(PartialEq)]
enum WizardType {
    Module,
    Testbench,
}

struct VerilogApp {
    state: AppState,
    splash_timer: f32,
    project: Option<Project>,
    
    // Editor State
    code: String,
    active_file: Option<PathBuf>,
    console_output: String,

    // File Wizard State (Inside Editor)
    show_file_wizard: bool,
    wiz_type: WizardType,
    wiz_mod_name: String,
    wiz_pins: Vec<(String, String)>,

    // Project Wizard State (Startup)
    new_proj_name: String,
    new_proj_path: Option<PathBuf>,
}

impl Default for VerilogApp {
    fn default() -> Self {
        Self {
            state: AppState::Splash,
            splash_timer: 0.0,
            project: None,
            code: String::new(),
            active_file: None,
            console_output: "VeriWaveX System Ready.\n".to_string(),
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
    fn get_tool_path(&self, tool_name: &str) -> PathBuf {
        let exe_ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
        let filename = format!("{}{}", tool_name, exe_ext);
        let mut root_dir = std::env::current_exe().unwrap();
        root_dir.pop();
        if root_dir.ends_with("debug") || root_dir.ends_with("release") {
            root_dir.pop(); root_dir.pop();
        }

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
            
            let proj = Project {
                name: self.new_proj_name.clone(),
                root: proj_dir.clone(),
                source_files: vec![],
            };
            
            let config_path = proj_dir.join(format!("{}.vwx", self.new_proj_name));
            let json = serde_json::to_string_pretty(&proj).unwrap();
            let _ = fs::write(config_path, json);
            
            self.project = Some(proj);
            self.state = AppState::Editor;
        }
    }

    fn open_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new().add_filter("VeriWaveX Project", &["vwx"]).pick_file() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(proj) = serde_json::from_str::<Project>(&data) {
                    self.project = Some(proj);
                    self.state = AppState::Editor;
                }
            }
        }
    }

    fn save_current_file(&mut self) {
        if let Some(path) = &self.active_file {
            let _ = fs::write(path, &self.code);
        }
    }

    fn run_sim(&mut self) {
        self.save_current_file();
        let Some(proj) = self.project.as_ref() else { return };
        
        let iverilog = self.get_tool_path("iverilog");
        let vvp = self.get_tool_path("vvp");
        let gtkwave = self.get_tool_path("gtkwave");
        let ivl_root = iverilog.parent().unwrap().parent().unwrap().join("lib/ivl");

        let mut args = vec!["-o".to_string(), "sim.vvp".to_string()];
        for f in &proj.source_files {
            if let Some(s) = f.to_str() { args.push(s.to_string()); }
        }

        let compile = Command::new(&iverilog)
            .env("IVL_ROOT", &ivl_root)
            .current_dir(&proj.root)
            .args(&args)
            .output();

        match compile {
            Ok(out) if out.status.success() => {
                let _ = Command::new(vvp).current_dir(&proj.root).arg("sim.vvp").output();
                let _ = Command::new(gtkwave).current_dir(&proj.root).arg("dump.vcd").spawn();
                self.console_output += "Simulation Success!\n";
            }
            Ok(out) => self.console_output = format!("Verilog Error:\n{}", String::from_utf8_lossy(&out.stderr)),
            Err(e) => self.console_output = format!("Tool Error: {}\n", e),
        }
    }
}

impl eframe::App for VerilogApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.state {
            AppState::Splash => {
                self.splash_timer += ui.input(|i| i.stable_dt);
                if self.splash_timer > 2.5 { self.state = AppState::Startup; }
                
                // Force black background for splash
                ui.painter().rect_filled(ui.max_rect(), 0.0, egui::Color32::BLACK);

                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(egui::include_image!("../assets/splash.png"))
                        .max_size(egui::vec2(600.0, 250.0))
                        .shrink_to_fit());
                });
                ui.ctx().request_repaint();
            }

            AppState::Startup => {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.add(egui::Image::new(egui::include_image!("../assets/logo.png")).max_width(120.0));
                    ui.heading(egui::RichText::new("VeriWaveX").size(40.0).strong());
                    ui.label("Verilog Simulation Suite");
                    ui.add_space(40.0);

                    if ui.add(egui::Button::new("🆕 Create New Project").min_size(egui::vec2(250.0, 50.0))).clicked() {
                        self.state = AppState::ProjectWizard;
                    }
                    ui.add_space(15.0);
                    if ui.add(egui::Button::new("📂 Open Existing Project").min_size(egui::vec2(250.0, 50.0))).clicked() {
                        self.open_project();
                    }
                });
            }

            AppState::ProjectWizard => {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Project Settings");
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() * 0.3);
                        ui.label("Project Name: ");
                        ui.text_edit_singleline(&mut self.new_proj_name);
                    });

                    ui.add_space(10.0);
                    if ui.button("📁 Select Save Location").clicked() {
                        self.new_proj_path = rfd::FileDialog::new().pick_folder();
                    }

                    if let Some(path) = &self.new_proj_path {
                        ui.label(format!("Path: {}", path.display()));
                        ui.add_space(20.0);
                        if ui.add(egui::Button::new("✅ Create Project").min_size(egui::vec2(150.0, 40.0))).clicked() {
                            self.finalize_project_creation();
                        }
                    }

                    ui.add_space(40.0);
                    if ui.button("⬅ Back").clicked() { self.state = AppState::Startup; }
                });
            }

            AppState::Editor => {
                // 1. Project Navigator
                egui::Panel::left("nav").default_size(200.0).show_inside(ui, |ui| {
                    ui.heading("📁 Navigator");
                    ui.separator();
                    if ui.button("➕ New File").clicked() { self.show_file_wizard = true; }
                    ui.add_space(10.0);

                    let files = self.project.as_ref().map(|p| p.source_files.clone()).unwrap_or_default();
                    for file in files {
                        let name = file.file_name().unwrap().to_string_lossy();
                        if ui.selectable_label(self.active_file.as_ref() == Some(&file), name).clicked() {
                            self.save_current_file();
                            if let Ok(content) = fs::read_to_string(&file) {
                                self.code = content;
                                self.active_file = Some(file);
                            }
                        }
                    }
                });

                // 2. Console
                egui::Panel::bottom("console").resizable(true).default_size(160.0).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Console");
                        if ui.button("🚀 Simulate").clicked() { self.run_sim(); }
                        if ui.button("💾 Save").clicked() { self.save_current_file(); }
                        if ui.button("🗑 Clear").clicked() { self.console_output.clear(); }
                    });
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        ui.add(egui::Label::new(&self.console_output).wrap());
                    });
                });

                // 3. Editor
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    if self.active_file.is_some() {
                        CodeEditor::default()
                            .id_source("v_editor")
                            .with_theme(ColorTheme::GRUVBOX)
                            .with_syntax(Syntax::rust())
                            .with_numlines(true)
                            .show(ui, &mut self.code);
                    } else {
                        ui.centered_and_justified(|ui| { ui.label("Select or create a file to start."); });
                    }
                });
            }
        }

        // --- File Wizard Window ---
        if self.show_file_wizard {
            egui::Window::new("New Verilog File").collapsible(false).show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.wiz_type, WizardType::Module, "Module");
                    ui.selectable_value(&mut self.wiz_type, WizardType::Testbench, "Testbench");
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.wiz_mod_name);
                });

                if self.wiz_type == WizardType::Module {
                    ui.separator();
                    let mut to_remove = None;
                    for (i, (pname, pdir)) in self.wiz_pins.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(pname);
                            egui::ComboBox::from_id_salt(i).selected_text(pdir.clone()).show_ui(ui, |ui| {
                                ui.selectable_value(pdir, "input".into(), "input");
                                ui.selectable_value(pdir, "output".into(), "output");
                            });
                            if ui.button("❌").clicked() { to_remove = Some(i); }
                        });
                    }
                    if let Some(i) = to_remove { self.wiz_pins.remove(i); }
                    if ui.button("➕ Add Pin").clicked() { self.wiz_pins.push(("new_pin".into(), "input".into())); }
                }

                if ui.button("✅ Create").clicked() {
                    let generated_code = match self.wiz_type {
                        WizardType::Module => {
                            let mut s = format!("module {} (\n", self.wiz_mod_name);
                            for (i, (n, d)) in self.wiz_pins.iter().enumerate() {
                                let comma = if i == self.wiz_pins.len() - 1 { "" } else { "," };
                                s.push_str(&format!("    {} {}{}\n", d, n, comma));
                            }
                            s.push_str(");\n\nendmodule"); s
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
                        let vwx = proj.root.join(format!("{}.vwx", proj.name));
                        let _ = fs::write(vwx, serde_json::to_string_pretty(proj).unwrap());
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
            .with_inner_size([1100.0, 750.0])
            .with_icon(icon.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native("VeriWaveX - beta", options, Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(VerilogApp::default()))
    }))
}