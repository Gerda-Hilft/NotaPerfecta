mod commands;
mod models;

use commands::{export, formcheck, ollama, pdf};
use models::{AiSuggestion, Correction, ExportResult};
use std::path::Path;

#[tauri::command]
fn read_pdf_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("Datei konnte nicht gelesen werden: {e}"))
}

#[tauri::command]
async fn extract_text_from_pdf(path: String) -> Result<String, String> {
    pdf::extract_text_from_pdf(path).await
}

#[tauri::command]
fn check_formvorschriften(path: String) -> Result<Vec<AiSuggestion>, String> {
    formcheck::check(&path)
}

#[tauri::command]
async fn check_spelling_ai(
    path: String,
    ollama_url: String,
    model_override: String,
) -> Result<Vec<AiSuggestion>, String> {
    ollama::check_spelling_ai(path, ollama_url, model_override).await
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
) -> Result<ExportResult, String> {
    export::export_corrected_pdf(original_path, accepted_corrections, output_dir).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WebKitGTK 2.42+ ships a DMABUF-based renderer that paints a blank white
    // window on many Linux GPU/driver/compositor combinations (NVIDIA proprietary
    // drivers, some Mesa/AMD/Intel setups, VMs, and bleeding-edge distros such as
    // Arch/CachyOS). The window and WebKit load fine, but the web content never
    // renders. Forcing the legacy renderer fixes the white screen. Only set it when
    // the user hasn't chosen a value so it stays overridable.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read_pdf_bytes,
            extract_text_from_pdf,
            check_formvorschriften,
            check_spelling_ai,
            list_ollama_models,
            list_pdf_files,
            copy_file,
            export_corrected_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
