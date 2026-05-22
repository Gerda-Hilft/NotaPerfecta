mod commands;
mod models;

use commands::{dictionary, export, ollama, pdf};
use models::{AiSuggestion, Correction, SpellingError};

#[tauri::command]
async fn extract_text_from_pdf(path: String) -> Result<String, String> {
    pdf::extract_text_from_pdf(path).await
}

#[tauri::command]
async fn check_spelling_dictionary(text: String) -> Result<Vec<SpellingError>, String> {
    dictionary::check_spelling_dictionary(text).await
}

#[tauri::command]
async fn check_spelling_ai(text: String) -> Result<Vec<AiSuggestion>, String> {
    ollama::check_spelling_ai(text).await
}

#[tauri::command]
async fn export_corrected_pdf(
    original_path: String,
    accepted_corrections: Vec<Correction>,
) -> Result<String, String> {
    export::export_corrected_pdf(original_path, accepted_corrections).await
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
            export_corrected_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
