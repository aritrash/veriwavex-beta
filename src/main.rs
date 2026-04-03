use eframe::egui;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Aether Verilog IDE",
        native_options,
        Box::new(|_cc| Box::new(VerilogApp::default())),
    )
}

struct VerilogApp {
    code: String,
    file_path: Option<PathBuf>,
    console_output: String,
}

impl Default for VerilogApp {
    fn default() -> Self {
        Self {
            code: "// Start your Verilog code here...\nmodule top();\n\nendmodule".to_string(),
            file_path: None,
            console_output: "System Ready. Use 'Save' before 'Simulate'.".to_string(),
        }
    }
}

impl VerilogApp {
    /// Helper to find the binaries in the vendor folder you created
    fn get_tool_path(&self, tool_name: &str) -> PathBuf {
        let exe_ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
        let filename = format!("{}{}", tool_name, exe_ext);

        // Precise mapping to your current vendor structure
        let local_path = match tool_name {
            "iverilog" | "vvp" => {
                std::env::current_dir().unwrap()
                    .join("vendor").join("iverilog").join(&filename)
            }
            "gtkwave" => {
                // Note the extra .join("bin") here to match your folder
                std::env::current_dir().unwrap()
                    .join("vendor").join("gtkwave").join("bin").join(&filename)
            }
            _ => PathBuf::from(&filename),
        };

        if local_path.exists() {
            return local_path;
        }

        // Fallback to your local installation if portable folder isn't found
        let fallback = if tool_name == "gtkwave" {
            Path::new(r"C:\iverilog\gtkwave\bin").join(&filename)
        } else {
            Path::new(r"C:\iverilog\bin").join(&filename)
        };

        if fallback.exists() { fallback } else { PathBuf::from(filename) }
    }

    fn save_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Verilog", &["v", "sv"])
            .save_file() 
        {
            if fs::write(&path, &self.code).is_ok() {
                // Option A: Use the info BEFORE moving 'path'
                let filename = path.file_name().unwrap().to_string_lossy().to_string();
                self.file_path = Some(path); 
                self.console_output = format!("Saved: {}", filename);
            }
        }
    }

    fn generate_uut(&mut self) {
        let re = Regex::new(r"module\s+(\w+)").unwrap();
        let module_name = re.captures(&self.code)
            .and_then(|cap| cap.get(1))
            .map_or("top", |m| m.as_str());

        let tb = format!(
            "\n\nmodule {0}_tb;\n  // Stimulus here\n  {0} uut ();\n\n  initial begin\n    $dumpfile(\"dump.vcd\");\n    $dumpvars(0, {0}_tb);\n    #100 $finish;\n  end\nendmodule",
            module_name
        );
        self.code.push_str(&tb);
    }

    fn run_simulation(&mut self) {
        let Some(path) = &self.file_path else {
            self.console_output = "Error: Please Save file first!".to_string();
            return;
        };

        let iverilog = self.get_tool_path("iverilog");
        let vvp = self.get_tool_path("vvp");
        let gtkwave = self.get_tool_path("gtkwave");

        self.console_output = "Compiling...".to_string();

        // 1. Compile
        let compile_res = Command::new(iverilog)
            .args(["-o", "sim.vvp", path.to_str().unwrap()])
            .output();

        match compile_res {
            Ok(out) if out.status.success() => {
                // 2. Run simulation (generates dump.vcd)
                let _ = Command::new(vvp).arg("sim.vvp").output();
                
                // 3. Launch Waveform Viewer
                let _ = Command::new(gtkwave).arg("dump.vcd").spawn();
                
                self.console_output = "Success! GTKWave launched.".to_string();
            }
            Ok(out) => {
                self.console_output = format!("Compile Error:\n{}", String::from_utf8_lossy(&out.stderr));
            }
            Err(e) => self.console_output = format!("Toolchain Error: {}", e),
        }
    }
}

impl eframe::App for VerilogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() { self.save_file(); }
                if ui.button("🛠 Gen UUT").clicked() { self.generate_uut(); }
                if ui.button("🚀 Simulate").clicked() { self.run_simulation(); }
                
                if let Some(p) = &self.file_path {
                    ui.label(format!("Active: {}", p.display()));
                }
            });
        });

        egui::TopBottomPanel::bottom("logs").resizable(true).show(ctx, |ui| {
            ui.label("Console Output:");
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(egui::Label::new(&self.console_output).wrap(true));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            CodeEditor::default()
                .id_source("v_editor")
                .with_rows(30)
                .with_fontsize(14.0)
                .with_theme(ColorTheme::GRUVBOX)
                .with_syntax(Syntax::rust()) // Proxy for Verilog
                .with_numlines(true)
                .show(ui, &mut self.code);
        });
    }
}