// File Path: /beam-patcher/src/preview.rs
// Purpose: Double-clickable GUI entry point for the isolated BEAM Preview
// Module: GUI - Windows
// Version: 0.1.0
// Description: Launch Tauri preview without writing a config or accessing a game client
// Dependency Layer: UI
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = beam_ui::run_preview() {
        eprintln!("BEAM GUI Preview failed: {error}");
        std::process::exit(1);
    }
}
