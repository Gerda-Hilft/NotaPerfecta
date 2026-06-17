mod commands;
mod models;

use commands::{dictionary, export, ollama, pdf};
use models::{AiSuggestion, Correction, SpellingError};
use std::path::Path;

#[tauri::command]
async fn extract_text_from_pdf(path: String) -> Result<String, String> {
    pdf::extract_text_from_pdf(path).await
}

#[tauri::command]
async fn check_spelling_dictionary(text: String) -> Result<Vec<SpellingError>, String> {
    dictionary::check_spelling_dictionary(text).await
}

#[tauri::command]
async fn check_spelling_ai(
    text: String,
    ollama_url: String,
    model_override: String,
) -> Result<Vec<AiSuggestion>, String> {
    ollama::check_spelling_ai(text, ollama_url, model_override).await
}

#[tauri::command]
fn list_pdf_files(directory: String) -> Result<Vec<String>, String> {
    let dir = Path::new(&directory);
    if !dir.is_dir() {
        return Err(format!("{directory} ist kein Verzeichnis"));
    }
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("Verzeichnis konnte nicht gelesen werden: {e}"))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    files.sort();
    Ok(files)
}

#[tauri::command]
async fn list_ollama_models(ollama_url: String) -> Result<Vec<String>, String> {
    ollama::list_ollama_models(ollama_url).await
}

#[tauri::command]
fn copy_file(source: String, dest_dir: String) -> Result<String, String> {
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Ordner konnte nicht erstellt werden: {e}"))?;
    let src = Path::new(&source);
    let filename = src
        .file_name()
        .ok_or_else(|| "Dateiname konnte nicht ermittelt werden".to_string())?;
    let dest = Path::new(&dest_dir).join(filename);
    std::fs::copy(src, &dest)
        .map_err(|e| format!("Datei konnte nicht kopiert werden: {e}"))?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
async fn export_corrected_pdf(
    original_path: String,
    accepted_corrections: Vec<Correction>,
    output_dir: Option<String>,
) -> Result<String, String> {
    export::export_corrected_pdf(original_path, accepted_corrections, output_dir).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            extract_text_from_pdf,
            check_spelling_dictionary,
            check_spelling_ai,
            list_ollama_models,
            list_pdf_files,
            copy_file,
            export_corrected_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
