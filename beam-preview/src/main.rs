// File Path: /beam-preview/src/main.rs
// Purpose: Isolated graphical BEAM preview (never opens the legacy Tauri web UI)
// Module: BEAM Desktop - Rust/Tauri
// Version: 0.1.0
// Description: Read-only launcher preview plus temporary THOR sandbox verification
// Dependency Layer: UI
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct PreviewStatus {
    name: &'static str,
    mode: &'static str,
    game_mutations_enabled: bool,
    launcher_enabled: bool,
}

#[derive(Serialize)]
struct ThorPreviewReport {
    installed_files: usize,
    removed_files: usize,
    no_grf_written: bool,
}

#[tauri::command]
fn preview_status() -> PreviewStatus {
    PreviewStatus {
        name: "LevelUpGT BEAM Patcher",
        mode: "Preview - Safe Mode",
        game_mutations_enabled: false,
        launcher_enabled: false,
    }
}

#[tauri::command]
async fn preview_verify_thor(path: String) -> Result<ThorPreviewReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let candidate = Path::new(&path);
        if !candidate.is_file() ||
            candidate.extension().and_then(|e| e.to_str())
                .map(|e| !e.eq_ignore_ascii_case("thor")).unwrap_or(true)
        {
            return Err("Yalnizca yerel .thor dosyalari secilebilir.".to_string());
        }
        let len = std::fs::metadata(candidate)
            .map_err(|e| format!("THOR dosyasi okunamadi: {e}"))?.len();
        if len == 0 || len > 512 * 1024 * 1024 {
            return Err("THOR dosyasi boyut siniri asildi.".to_string());
        }
        let result = beam_core::patcher::smoke_test_loose_thor(candidate)
            .map_err(|e| format!("THOR test edilemedi: {e}"))?;
        Ok(ThorPreviewReport {
            installed_files: result.verified_files,
            removed_files: result.deleted_files,
            no_grf_written: result.no_grf_written,
        })
    }).await.map_err(|e| format!("THOR test islemi sonlandi: {e}"))?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            preview_status,
            preview_verify_thor
        ])
        .run(tauri::generate_context!())
        .expect("Unable to open isolated BEAM GUI Preview window");
}
