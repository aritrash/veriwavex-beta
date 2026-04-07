/*
 * VeriWaveX - Models & Data Structures
 * File: src/models.rs
 * Version: 2.0.0
 * Copyright (c) 2026 Aritrash Sarkar.
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The various visual states the application can be in.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum AppState {
    Splash,
    Startup,
    ProjectWizard,
    Editor,
}

/// User preference for the application's visual theme.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum AppTheme {
    System,
    Light,
    Dark,
}

/// The type of Verilog file the New File Wizard should generate.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum WizardType {
    Module,
    Testbench,
}

/// Represents a VeriWaveX project (.vwx).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub root: PathBuf,
    pub source_files: Vec<PathBuf>,
}

/// Persistent user settings stored in settings.json.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserSettings {
    pub recent_projects: Vec<PathBuf>,
}