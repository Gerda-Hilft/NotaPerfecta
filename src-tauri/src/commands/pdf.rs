use std::process::Command;

pub async fn extract_text_from_pdf(path: String) -> Result<String, String> {
    let result = Command::new("pdftotext")
        .args(["-nopgbrk", &path, "-"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout)
                .map_err(|e| format!("PDF-Text konnte nicht dekodiert werden: {e}"))
        }
        Ok(output) => Err(format!(
            "pdftotext fehlgeschlagen: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(_) => {
            // pdftotext not available, fall back to pdf-extract
            pdf_extract::extract_text(&path)
                .map_err(|e| format!("PDF konnte nicht gelesen werden: {e}"))
        }
    }
}
