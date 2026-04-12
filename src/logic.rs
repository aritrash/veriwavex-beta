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
    let mut root_dir = std::env::current_exe().unwrap();
    root_dir.pop(); // Remove binary name

    if root_dir.ends_with("debug") || root_dir.ends_with("release") {
        root_dir.pop(); // Remove 'debug'
        root_dir.pop(); // Remove 'target'
    }

    let subfolder = if cfg!(target_os = "linux") {
        match tool_name {
            "yosys"            => "vendor/yosys/bin",
            "iverilog" | "vvp" => "vendor/iverilog/bin",
            "gtkwave" => "vendor/iverilog/gtkwave/bin",
            "dot"              => "vendor/graphviz/bin",
            _                  => "vendor/yosys/bin",
        }
    } else {
        match tool_name {
            "iverilog" | "vvp" => "vendor/iverilog/bin",
            "gtkwave" => "vendor/iverilog/gtkwave/bin",
            "yosys"            => "vendor/yosys/bin",
            "dot"              => "vendor/graphviz/bin",
            _                  => "",
        }
    };

    let filename = if cfg!(target_os = "linux") {
        tool_name.to_string()
    } else {
        format!("{}.exe", tool_name)
    };

    let full_path = root_dir.join(subfolder).join(filename);
    
    // --- THE TRACER ---
    println!("🔍 DEBUG: Looking for {} at: {:?}", tool_name, full_path);
    
    full_path
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
    
    // Get the directory containing the verilog file (the project root)
    let proj_root = verilog_file.parent().unwrap_or(std::path::Path::new("."));
    let file_name = verilog_file.file_name().unwrap().to_string_lossy();
    let file_stem = verilog_file.file_stem().unwrap().to_string_lossy();
    
    let dot_file = proj_root.join(format!("{}.dot", file_stem));
    let png_file = proj_root.join(format!("{}.png", file_stem));

    let mut yosys_cmd = Command::new(&yosys_exe);
    
    // 1. SET THE ENVIRONMENT
    if let Some(bin_dir) = yosys_exe.parent() {
        // Prepend Yosys bin to PATH so it finds yosys-abc.exe and its DLLs
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut new_path = bin_dir.to_path_buf().into_os_string();
        new_path.push(";");
        new_path.push(current_path);
        yosys_cmd.env("PATH", new_path);

        // Tell Yosys where the 'share' folder is
        if let Some(yosys_home) = bin_dir.parent() {
            yosys_cmd.env("YOSYS_DATADIR", yosys_home.join("share/yosys"));
        }
    }

    // 2. RUN FROM PROJECT ROOT
    yosys_cmd.current_dir(proj_root);

    // Use relative filename and prefix to avoid Windows path escaping issues
    let yosys_script = format!(
        "read_verilog {}; hierarchy -check; proc; opt; show -format dot -prefix {}",
        file_name, file_stem
    );

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        yosys_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let yosys_output = yosys_cmd.arg("-p").arg(yosys_script).output();

    match yosys_output {
        Ok(out) if out.status.success() => {
            // --- GRAPHVIZ PHASE ---
            let mut dot_cmd = Command::new(&dot_exe);
            
            if let Some(bin_dir) = dot_exe.parent() {
                let current_path = std::env::var_os("PATH").unwrap_or_default();
                let mut new_path = bin_dir.to_path_buf().into_os_string();
                new_path.push(";");
                new_path.push(current_path);
                dot_cmd.env("PATH", new_path);
            }

            dot_cmd.current_dir(proj_root);
            
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                dot_cmd.creation_flags(0x08000000);
            }

            let dot_output = dot_cmd
                .args(["-Tpng", &format!("{}.dot", file_stem), "-o", &format!("{}.png", file_stem)])
                .output();

            if let Ok(out) = dot_output {
                if out.status.success() { return Ok(png_file); }
                return Err(format!("Graphviz Error: {}", String::from_utf8_lossy(&out.stderr)));
            }
            Err("Graphviz failed to generate image.".into())
        }
        Ok(out) => Err(format!("YOSYS ERROR:\n{}", String::from_utf8_lossy(&out.stdout))),
        Err(e) => Err(format!("Failed to start Yosys: {}", e)),
    }
}