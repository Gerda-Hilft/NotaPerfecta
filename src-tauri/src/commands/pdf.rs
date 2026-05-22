pub async fn extract_text_from_pdf(path: String) -> Result<String, String> {
    pdf_extract::extract_text(path).map_err(|e| format!("PDF konnte nicht gelesen werden: {e}"))
}
