/*
 * VeriWaveX - Logic Engine
 * File: src/logic.rs
 * Version: 2.0.0
 * Copyright (c) 2026 Aritrash Sarkar.
 */

use std::fs;
use std::path::PathBuf;
use egui_code_editor::Syntax;
use crate::models::{Project, UserSettings};
use std::process::Command;

// --- 1. SETTINGS & PERSISTENCE ---

pub fn load_settings() -> UserSettings {
    fs::read_to_string("settings.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &UserSettings) {
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write("settings.json", json);
    }
}

pub fn add_to_recent(settings: &mut UserSettings, path: PathBuf) {
    settings.recent_projects.retain(|p| p != &path);
    settings.recent_projects.insert(0, path);
    if settings.recent_projects.len() > 5 {
        settings.recent_projects.pop();
    }
    save_settings(settings);
}

// --- 2. TOOLCHAIN & PATH DETECTION ---

pub fn get_tool_path(tool_name: &str) -> PathBuf {
    if cfg!(target_os = "linux") {
        return which::which(tool_name).unwrap_or_else(|_| PathBuf::from(tool_name));
    }

    let exe_ext = ".exe";
    let filename = format!("{}{}", tool_name, exe_ext);
    
    let mut root_dir = std::env::current_exe().unwrap();
    root_dir.pop(); 
    if root_dir.ends_with("debug") || root_dir.ends_with("release") {
        root_dir.pop(); root_dir.pop(); 
    }

    let subfolder = match tool_name {
        "iverilog" | "vvp" => "vendor/iverilog/bin",
        "gtkwave"          => "vendor/iverilog/gtkwave/bin", 
        "yosys"            => "vendor/yosys/bin",
        "dot"              => "vendor/graphviz/bin",
        _                  => "",
    };

    let full_path = root_dir.join(subfolder).join(&filename);
    full_path.canonicalize().unwrap_or(full_path)
}

// --- 3. VERILOG SYNTAX DEFINITION ---

pub fn verilog_syntax() -> Syntax {
    Syntax {
        language: "Verilog",
        case_sensitive: true,
        comment: "//",
        comment_multiline: ["/*", "*/"],
        // Using collect() directly on string literals
        keywords: vec![
            "module", "endmodule", "input", "output", "inout", "reg", "wire",
            "assign", "always", "initial", "begin", "end", "posedge", "negedge",
            "if", "else", "case", "endcase", "default", "parameter", "localparam", 
            "generate", "genvar"
        ].into_iter().collect(),
        types: vec!["wire", "reg", "integer", "time", "real"].into_iter().collect(),
        special: vec!["$display", "$finish", "$dumpfile", "$dumpvars", "$time", "$stop", "$monitor"].into_iter().collect(),
        hyperlinks: std::collections::BTreeSet::new(),
        quotes: std::collections::BTreeSet::from(['"']), 
    }
}

// --- 4. LEGACY COMPATIBILITY ---

pub fn import_xilinx_ise(path: PathBuf) -> Result<Project, String> {
    let xml_content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let proj_name = path.file_stem().unwrap().to_string_lossy().to_string();
    let root = path.parent().unwrap().to_path_buf();
    let mut source_files = Vec::new();

    // Look for <file xil_pn:name="filename.v" ... />
    for line in xml_content.lines() {
        if line.contains("FILE_VERILOG") || line.contains(".v\"") {
            if let Some(start) = line.find("xil_pn:name=\"") {
                let rest = &line[start + 13..];
                if let Some(end) = rest.find('\"') {
                    let filename = &rest[..end];
                    let full_path = root.join(filename);
                    if full_path.exists() {
                        source_files.push(full_path);
                    }
                }
            }
        }
    }

    if source_files.is_empty() {
        return Err("No Verilog files found in ISE project.".into());
    }

    Ok(Project {
        name: proj_name,
        root,
        source_files,
    })
}

/// Stub for importing Xilinx PlanAhead Projects (.ppr is XML-based)
pub fn import_planahead(_path: PathBuf) -> Result<Project, String> {
    // TODO: Implement XML Parser for .ppr
    Err("PlanAhead Import Logic currently in development for v2.0.0".to_string())
}

pub fn generate_schematic(verilog_file: &PathBuf) -> Result<PathBuf, String> {
    let yosys_exe = get_tool_path("yosys");
    let dot_exe = get_tool_path("dot");
    
    let dot_file = verilog_file.with_extension("dot");
    let png_file = verilog_file.with_extension("png");

    // 1. Setup Yosys Command
    let mut yosys_cmd = Command::new(&yosys_exe);
    
    if cfg!(target_os = "windows") {
        if let Some(bin_dir) = yosys_exe.parent() {
            // A. Help Yosys find yosys-abc.exe
            yosys_cmd.env("PATH", bin_dir); 
            
            // B. Help Yosys find its techlibs (the share folder)
            // We assume share is at ../share relative to bin/yosys.exe
            if let Some(yosys_root) = bin_dir.parent() {
                let share_path = yosys_root.join("share/yosys");
                yosys_cmd.env("YOSYS_DATADIR", share_path);
            }
        }
    }

    // Use a slightly more verbose script to catch errors early
    let yosys_script = format!(
        "read_verilog {:?}; hierarchy -check; proc; opt; write_graphviz {:?}",
        verilog_file, dot_file
    );

    let yosys_output = yosys_cmd.arg("-p").arg(yosys_script).output();

    match yosys_output {
        Ok(out) if out.status.success() => {
            // 2. Run Graphviz DOT
            let mut dot_cmd = Command::new(&dot_exe);
            if let Some(bin_dir) = dot_exe.parent() {
                dot_cmd.env("PATH", bin_dir);
            }

            let dot_status = dot_cmd
                .args(["-Tpng", dot_file.to_str().unwrap(), "-o", png_file.to_str().unwrap()])
                .output();

            if let Ok(out) = dot_status {
                if out.status.success() { return Ok(png_file); }
                return Err(format!("Graphviz Error: {}", String::from_utf8_lossy(&out.stderr)));
            }
            Err("Graphviz failed to generate image.".into())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            // If it's still empty, it's a library/path issue
            if stderr.is_empty() && stdout.is_empty() {
                return Err("Yosys crashed silently. Check if 'vendor/yosys/share' folder exists.".into());
            }
            Err(format!("YOSYS ERROR:\n{}\n{}", stdout, stderr))
        }
        Err(e) => Err(format!("Failed to start Yosys: {}", e)),
    }
}