use crate::models::Correction;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

pub async fn export_corrected_pdf(
    original_path: String,
    accepted_corrections: Vec<Correction>,
) -> Result<String, String> {
    let original_text = pdf_extract::extract_text(&original_path)
        .map_err(|e| format!("Original-PDF konnte nicht gelesen werden: {e}"))?;

    let mut corrected = original_text;
    for correction in accepted_corrections {
        corrected = corrected.replacen(&correction.original, &correction.correction, 1);
    }

    let mut target = PathBuf::from(original_path);
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("zeugnis")
        .to_string();
    target.set_file_name(format!("{stem}_korrigiert.pdf"));

    let (doc, page1, layer1) = PdfDocument::new("Korrigiertes Zeugnis", Mm(210.0), Mm(297.0), "Ebene 1");
    let layer = doc.get_page(page1).get_layer(layer1);
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Schrift konnte nicht geladen werden: {e}"))?;

    let mut y = 280.0;
    for line in corrected.lines() {
        layer.use_text(line, 11.0, Mm(12.0), Mm(y), &font);
        y -= 6.0;
        if y < 12.0 {
            break;
        }
    }

    let file = File::create(&target).map_err(|e| format!("Datei konnte nicht erzeugt werden: {e}"))?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| format!("PDF konnte nicht gespeichert werden: {e}"))?;

    Ok(target.to_string_lossy().to_string())
}
