// File Path: /beam-ui/src/preview.rs
// Purpose: Real Tauri desktop window for safe, non-mutating BEAM preview
// Module: UI - Tauri 1.x
// Version: 0.1.0
// Description: THOR sandbox test only; no game install, launch or self-update
// Dependency Layer: UI
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct PreviewStatus {
    name: &'static str,
    mode: &'static str,
    engine: &'static str,
    game_mutations_enabled: bool,
    launcher_enabled: bool,
}

#[derive(Serialize)]
pub struct ThorPreviewReport {
    installed_files: usize,
    removed_files: usize,
    no_grf_written: bool,
}

#[tauri::command]
fn preview_status() -> PreviewStatus {
    PreviewStatus {
        name: "LevelUpGT BEAM Patcher",
        mode: "GUI Preview - Safe",
        engine: "BEAM Rust THOR",
        game_mutations_enabled: false,
        launcher_enabled: false,
    }
}

#[tauri::command]
async fn preview_verify_thor(path: String) -> std::result::Result<ThorPreviewReport, String> {
    // The frontend selects a local .thor file. It never passes a game path.
    tauri::async_runtime::spawn_blocking(move || {
        let candidate = Path::new(&path);
        if !candidate.is_file()
            || candidate.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| !ext.eq_ignore_ascii_case("thor"))
                .unwrap_or(true)
        {
            return Err("Yalnizca yerel .thor dosyasi secilebilir.".to_string());
        }
        let len = std::fs::metadata(candidate)
            .map_err(|e| format!("THOR okunamadi: {e}"))?
            .len();
        if len == 0 || len > 512 * 1024 * 1024 {
            return Err("THOR boyutu test sinirlarinin disinda.".to_string());
        }
        let result = beam_core::patcher::smoke_test_loose_thor(candidate)
            .map_err(|e| format!("THOR test edilemedi: {e}"))?;
        Ok(ThorPreviewReport {
            installed_files: result.verified_files,
            removed_files: result.deleted_files,
            no_grf_written: result.no_grf_written,
        })
    })
    .await
    .map_err(|e| format!("Test islemi sonlandirildi: {e}"))?
}

pub fn run_preview() -> Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            preview_status,
            preview_verify_thor,
        ])
        .run(tauri::generate_context!("tauri.preview.conf.json"))
        .expect("BEAM Preview Tauri window failed to start");
    Ok(())
}
