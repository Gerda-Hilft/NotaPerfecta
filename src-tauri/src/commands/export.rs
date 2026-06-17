use crate::models::Correction;
use lopdf::content::Content;
use lopdf::{Document, Object};
use std::path::PathBuf;

fn pdf_bytes_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

fn string_to_pdf_bytes(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| if (c as u32) <= 255 { c as u8 } else { b'?' })
        .collect()
}

fn try_replace(text: &str, corrections: &mut Vec<(String, String)>) -> Option<String> {
    let mut result = text.to_string();
    let mut changed = false;
    corrections.retain(|(original, correction)| {
        if let Some(pos) = result.find(original.as_str()) {
            result.replace_range(pos..pos + original.len(), correction);
            changed = true;
            false
        } else {
            true
        }
    });
    if changed {
        Some(result)
    } else {
        None
    }
}

fn apply_corrections(
    operations: &mut [lopdf::content::Operation],
    corrections: &mut Vec<(String, String)>,
) -> bool {
    let mut any_changed = false;

    for op in operations.iter_mut() {
        if corrections.is_empty() {
            break;
        }

        match op.operator.as_str() {
            "Tj" | "'" => {
                if let Some(Object::String(ref bytes, fmt)) = op.operands.first().cloned() {
                    let text = pdf_bytes_to_string(&bytes);
                    if let Some(new_text) = try_replace(&text, corrections) {
                        op.operands[0] = Object::String(string_to_pdf_bytes(&new_text), fmt);
                        any_changed = true;
                    }
                }
            }
            "TJ" => {
                if let Some(Object::Array(ref arr)) = op.operands.first().cloned() {
                    let mut new_arr = arr.clone();
                    let mut arr_changed = false;

                    for obj in new_arr.iter_mut() {
                        if corrections.is_empty() {
                            break;
                        }
                        if let Object::String(ref bytes, fmt) = obj.clone() {
                            let text = pdf_bytes_to_string(&bytes);
                            if let Some(new_text) = try_replace(&text, corrections) {
                                *obj = Object::String(string_to_pdf_bytes(&new_text), fmt);
                                arr_changed = true;
                            }
                        }
                    }

                    if !arr_changed && !corrections.is_empty() {
                        let parts: Vec<_> = arr
                            .iter()
                            .filter_map(|obj| {
                                if let Object::String(ref bytes, _) = obj {
                                    Some(pdf_bytes_to_string(bytes))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let full_text: String = parts.join("");

                        let first_fmt = arr
                            .iter()
                            .find_map(|obj| {
                                if let Object::String(_, fmt) = obj {
                                    Some(fmt.clone())
                                } else {
                                    None
                                }
                            });

                        if let (Some(new_text), Some(fmt)) =
                            (try_replace(&full_text, corrections), first_fmt)
                        {
                            new_arr = vec![Object::String(string_to_pdf_bytes(&new_text), fmt)];
                            arr_changed = true;
                        }
                    }

                    if arr_changed {
                        op.operands[0] = Object::Array(new_arr);
                        any_changed = true;
                    }
                }
            }
            _ => {}
        }
    }

    any_changed
}

fn correct_document(
    original_path: &str,
    accepted_corrections: Vec<Correction>,
    target: &PathBuf,
) -> Result<String, String> {
    let mut doc = Document::load(original_path)
        .map_err(|e| format!("PDF konnte nicht geladen werden: {e}"))?;

    let mut corrections: Vec<(String, String)> = accepted_corrections
        .into_iter()
        .map(|c| (c.original, c.correction))
        .collect();

    let page_ids: Vec<_> = doc.get_pages().values().cloned().collect();

    for page_id in page_ids {
        if corrections.is_empty() {
            break;
        }

        let content_stream_ids = {
            let page_dict = doc
                .get_dictionary(page_id)
                .map_err(|e| format!("Seite konnte nicht gelesen werden: {e}"))?;

            match page_dict.get(b"Contents") {
                Ok(Object::Reference(id)) => vec![*id],
                Ok(Object::Array(arr)) => arr
                    .iter()
                    .filter_map(|obj| {
                        if let Object::Reference(id) = obj {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => continue,
            }
        };

        for stream_id in content_stream_ids {
            if corrections.is_empty() {
                break;
            }

            let obj = match doc.get_object_mut(stream_id) {
                Ok(o) => o,
                _ => continue,
            };

            if let Object::Stream(ref mut stream) = obj {
                stream.decompress();

                let content = Content::decode(&stream.content)
                    .map_err(|e| format!("PDF-Inhalt konnte nicht dekodiert werden: {e}"))?;

                let mut operations = content.operations;
                let changed = apply_corrections(&mut operations, &mut corrections);

                if changed {
                    let new_content = Content { operations };
                    stream.content = new_content
                        .encode()
                        .map_err(|e| format!("PDF-Inhalt konnte nicht kodiert werden: {e}"))?;
                }
            }
        }
    }

    doc.save(target)
        .map_err(|e| format!("PDF konnte nicht gespeichert werden: {e}"))?;

    Ok(target.to_string_lossy().to_string())
}

pub async fn export_corrected_pdf(
    original_path: String,
    accepted_corrections: Vec<Correction>,
    output_dir: Option<String>,
) -> Result<String, String> {
    let target = if let Some(dir) = output_dir {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Ordner konnte nicht erstellt werden: {e}"))?;
        let filename = PathBuf::from(&original_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "korrigiert.pdf".into());
        PathBuf::from(dir).join(filename)
    } else {
        let mut t = PathBuf::from(&original_path);
        let stem = t
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("zeugnis")
            .to_string();
        t.set_file_name(format!("{stem}_korrigiert.pdf"));
        t
    };

    correct_document(&original_path, accepted_corrections, &target)
}
