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
            pdf_extract::extract_text(&path)
                .map_err(|e| format!("PDF konnte nicht gelesen werden: {e}"))
        }
    }
}

pub fn extract_text_layout(path: &str) -> Result<String, String> {
    let output = Command::new("pdftotext")
        .args(["-layout", "-nopgbrk", path, "-"])
        .output()
        .map_err(|e| format!("pdftotext nicht verfügbar: {e}"))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| format!("PDF-Text konnte nicht dekodiert werden: {e}"))
    } else {
        Err(format!(
            "pdftotext fehlgeschlagen: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
