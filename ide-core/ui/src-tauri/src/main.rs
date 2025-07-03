// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::File;
use std::io::Write;
use tauri::command;
use editor::syntax::{HighlightSpan, SupportedLanguage, SyntaxEngine};

#[command]
fn save_buffer(contents: String) -> Result<(), String> {
    let mut file = File::create("buffer.txt").map_err(|e| e.to_string())?;
    file.write_all(contents.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
fn load_buffer() -> Result<String, String> {
    std::fs::read_to_string("buffer.txt").map_err(|e| e.to_string())
}

#[command]
fn get_highlights(contents: String, language: String) -> Result<Vec<HighlightSpan>, String> {
    let lang = SupportedLanguage::from_extension(&language)
        .ok_or_else(|| format!("Unsupported language: {}", language))?;

    let mut engine = SyntaxEngine::new(lang);
    Ok(engine.extract_highlights(&contents))
}

#[command]
fn list_files(root: String) -> Result<Vec<String>, String> {
    let paths = std::fs::read_dir(root)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path().display().to_string())
        .collect();
    Ok(paths)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_buffer,
            load_buffer,
            get_highlights,
            list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}