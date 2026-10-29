//! The launcher's commands. Each one is a thin call into `chronicle-launcher-core`.

use chronicle_launcher_core::{library, paths};
use std::path::PathBuf;

#[tauri::command]
fn library() -> Result<Vec<library::GameView>, String> {
    library::library(&paths::root()).map_err(|e| e.to_string())
}

#[tauri::command]
fn choose_build(id: String, folder: PathBuf) -> Result<library::GameView, String> {
    library::choose_build(&paths::root(), &id, &folder)
}

#[tauri::command]
fn forget_build(id: String) -> Result<library::GameView, String> {
    library::forget_build(&paths::root(), &id)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![library, choose_build, forget_build])
        .run(tauri::generate_context!())
        .expect("the launcher's window could not start");
}
