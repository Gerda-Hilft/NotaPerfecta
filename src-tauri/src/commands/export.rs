use crate::models::Correction;
use lopdf::content::Content;
use lopdf::{Document, Object, ObjectId};
use std::collections::HashMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// FontCodec: bidirectional glyph-ID ↔ Unicode mapping
// ---------------------------------------------------------------------------

enum FontCodec {
    TwoByteMap {
        forward: HashMap<u16, String>,
        reverse: HashMap<String, u16>,
    },
    Passthrough,
}

impl FontCodec {
    fn decode(&self, bytes: &[u8]) -> String {
        match self {
            Self::TwoByteMap { forward, .. } => bytes
                .chunks(2)
                .map(|chunk| {
                    let code = if chunk.len() == 2 {
                        (chunk[0] as u16) << 8 | chunk[1] as u16
                    } else {
                        chunk[0] as u16
                    };
                    forward
                        .get(&code)
                        .map(|s| s.as_str())
                        .unwrap_or("\u{FFFD}")
                })
                .collect(),
            Self::Passthrough => bytes.iter().map(|&b| b as char).collect(),
        }
    }

    fn encode(&self, text: &str) -> Option<Vec<u8>> {
        match self {
            Self::TwoByteMap { reverse, .. } => {
                let mut out = Vec::with_capacity(text.len() * 2);
                let mut remaining = text;
                while !remaining.is_empty() {
                    // Try longest match first (ligatures like "fi")
                    let mut matched = false;
                    let max_len = remaining.chars().count().min(4);
                    for len in (1..=max_len).rev() {
                        let prefix: String = remaining.chars().take(len).collect();
                        if let Some(&code) = reverse.get(&prefix) {
                            out.push((code >> 8) as u8);
                            out.push((code & 0xFF) as u8);
                            remaining = &remaining[prefix.len()..];
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        return None;
                    }
                }
                Some(out)
            }
            Self::Passthrough => Some(
                text.chars()
                    .map(|c| if (c as u32) <= 255 { c as u8 } else { b'?' })
                    .collect(),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Minimal ToUnicode CMap parser
// ---------------------------------------------------------------------------

fn parse_hex(s: &str) -> Option<u16> {
    u16::from_str_radix(s, 16).ok()
}

fn parse_tounicode_cmap(data: &[u8]) -> HashMap<u16, String> {
    let text = String::from_utf8_lossy(data);
    let mut map = HashMap::new();

    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.ends_with("beginbfchar") {
            for char_line in lines.by_ref() {
                let char_trimmed = char_line.trim();
                if char_trimmed.starts_with("endbfchar") {
                    break;
                }
                let hexes: Vec<&str> = char_trimmed
                    .split('<')
                    .filter_map(|s| s.trim().strip_suffix('>'))
                    .collect();
                if hexes.len() >= 2 {
                    if let Some(src) = parse_hex(hexes[0]) {
                        let decoded = hex_to_string(hexes[1]);
                        if !decoded.is_empty() {
                            map.insert(src, decoded);
                        }
                    }
                }
            }
        } else if trimmed.ends_with("beginbfrange") {
            for range_line in lines.by_ref() {
                let range_trimmed = range_line.trim();
                if range_trimmed.starts_with("endbfrange") {
                    break;
                }
                let hexes: Vec<&str> = range_trimmed
                    .split('<')
                    .filter_map(|s| s.trim().strip_suffix('>'))
                    .collect();
                if hexes.len() >= 3 {
                    if let (Some(lo), Some(hi), Some(dst_start)) =
                        (parse_hex(hexes[0]), parse_hex(hexes[1]), parse_hex(hexes[2]))
                    {
                        for code in lo..=hi {
                            let cp = dst_start + (code - lo);
                            if let Some(ch) = char::from_u32(cp as u32) {
                                map.insert(code, ch.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    map
}

fn hex_to_string(hex: &str) -> String {
    // UTF-16BE encoded: pairs of 4 hex digits
    let mut result = String::new();
    let mut i = 0;
    while i + 4 <= hex.len() {
        if let Ok(cp) = u16::from_str_radix(&hex[i..i + 4], 16) {
            if let Some(ch) = char::from_u32(cp as u32) {
                result.push(ch);
            }
        }
        i += 4;
    }
    result
}

// ---------------------------------------------------------------------------
// Build codecs for all fonts on a page
// ---------------------------------------------------------------------------

fn build_page_codecs(doc: &Document, page_id: ObjectId) -> HashMap<Vec<u8>, FontCodec> {
    let mut codecs = HashMap::new();

    let fonts = match doc.get_page_fonts(page_id) {
        Ok(f) => f,
        Err(_) => return codecs,
    };

    for (name, font_dict) in &fonts {
        let codec = build_font_codec(font_dict, doc);
        codecs.insert(name.clone(), codec);
    }

    codecs
}

fn build_font_codec(font_dict: &lopdf::Dictionary, doc: &Document) -> FontCodec {
    let to_unicode = font_dict
        .get_deref(b"ToUnicode", doc)
        .ok()
        .and_then(|obj| obj.as_stream().ok());

    if let Some(stream) = to_unicode {
        let content = match stream.get_plain_content() {
            Ok(c) => c,
            Err(_) => return FontCodec::Passthrough,
        };

        let forward = parse_tounicode_cmap(&content);
        if forward.is_empty() {
            return FontCodec::Passthrough;
        }

        let reverse: HashMap<String, u16> =
            forward.iter().map(|(k, v)| (v.clone(), *k)).collect();
        FontCodec::TwoByteMap { forward, reverse }
    } else {
        FontCodec::Passthrough
    }
}

// ---------------------------------------------------------------------------
// Text replacement
// ---------------------------------------------------------------------------

/// Large negative kerning in a TJ array acts as a word space.
/// Threshold chosen from real Zeugnis PDFs: letter kerning ≈ -18…-45,
/// word gaps ≈ -200…-350.
const WORD_SPACE_THRESHOLD: f64 = -150.0;

fn kerning_value(obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(n) => Some(*n as f64),
        Object::Real(n) => Some(*n as f64),
        _ => None,
    }
}

fn concat_tj_with_spaces(arr: &[Object], codec: &FontCodec) -> String {
    let mut parts = Vec::new();
    let mut pending_space = false;

    for obj in arr {
        if let Object::String(ref bytes, _) = obj {
            if pending_space && !parts.is_empty() {
                parts.push(" ".to_string());
            }
            pending_space = false;
            parts.push(codec.decode(bytes));
        } else if let Some(kern) = kerning_value(obj) {
            if kern < WORD_SPACE_THRESHOLD {
                pending_space = true;
            }
        }
    }

    parts.join("")
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
    codecs: &HashMap<Vec<u8>, FontCodec>,
) -> bool {
    let mut any_changed = false;
    let mut current_font: Vec<u8> = Vec::new();
    let passthrough = FontCodec::Passthrough;

    for op in operations.iter_mut() {
        if corrections.is_empty() {
            break;
        }

        match op.operator.as_str() {
            "Tf" => {
                if let Some(Object::Name(ref name)) = op.operands.first() {
                    current_font = name.clone();
                }
            }
            "Tj" | "'" => {
                let codec = codecs.get(&current_font).unwrap_or(&passthrough);
                if let Some(Object::String(ref bytes, fmt)) = op.operands.first().cloned() {
                    let text = codec.decode(&bytes);
                    if let Some(new_text) = try_replace(&text, corrections) {
                        if let Some(new_bytes) = codec.encode(&new_text) {
                            op.operands[0] = Object::String(new_bytes, fmt);
                            any_changed = true;
                        }
                    }
                }
            }
            "TJ" => {
                let codec = codecs.get(&current_font).unwrap_or(&passthrough);
                if let Some(Object::Array(ref arr)) = op.operands.first().cloned() {
                    let mut new_arr = arr.clone();
                    let mut arr_changed = false;

                    // First pass: try replacing within individual string fragments
                    for obj in new_arr.iter_mut() {
                        if corrections.is_empty() {
                            break;
                        }
                        if let Object::String(ref bytes, fmt) = obj.clone() {
                            let text = codec.decode(&bytes);
                            if let Some(new_text) = try_replace(&text, corrections) {
                                if let Some(new_bytes) = codec.encode(&new_text) {
                                    *obj = Object::String(new_bytes, fmt);
                                    arr_changed = true;
                                }
                            }
                        }
                    }

                    // Second pass: concatenate all fragments with kerning-based
                    // word spaces so the text matches what pdftotext produces
                    if !arr_changed && !corrections.is_empty() {
                        let full_text = concat_tj_with_spaces(arr, codec);

                        let first_fmt = arr.iter().find_map(|obj| {
                            if let Object::String(_, fmt) = obj {
                                Some(fmt.clone())
                            } else {
                                None
                            }
                        });

                        if let (Some(new_text), Some(fmt)) =
                            (try_replace(&full_text, corrections), first_fmt)
                        {
                            if let Some(new_bytes) = codec.encode(&new_text) {
                                new_arr = vec![Object::String(new_bytes, fmt)];
                                arr_changed = true;
                            }
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

// ---------------------------------------------------------------------------
// Document-level correction
// ---------------------------------------------------------------------------

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

        // Build codecs while doc is immutably borrowed
        let codecs = build_page_codecs(&doc, page_id);

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
                let changed = apply_corrections(&mut operations, &mut corrections, &codecs);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Correction;

    #[test]
    fn cmap_parser_handles_bfchar_and_bfrange() {
        let cmap = b"
/CMapName /Adobe-Identity-UCS def
1 begincodespacerange
<0000> <FFFF>
endcodespacerange
2 beginbfchar
<0004> <0020>
<00A6> <00660069>
endbfchar
2 beginbfrange
<0045> <0054> <0061>
<0056> <0059> <0072>
endbfrange
endcmap
";
        let map = parse_tounicode_cmap(cmap);
        assert_eq!(map.get(&0x0004), Some(&" ".to_string()));
        assert_eq!(map.get(&0x00A6), Some(&"fi".to_string()));
        assert_eq!(map.get(&0x0045), Some(&"a".to_string()));
        assert_eq!(map.get(&0x0054), Some(&"p".to_string()));
        assert_eq!(map.get(&0x0056), Some(&"r".to_string()));
        assert_eq!(map.get(&0x0059), Some(&"u".to_string()));
    }

    #[test]
    fn codec_round_trip() {
        let mut forward = HashMap::new();
        forward.insert(0x0045u16, "a".to_string());
        forward.insert(0x0046, "b".to_string());
        forward.insert(0x0047, "c".to_string());
        forward.insert(0x00A6, "fi".to_string());
        let reverse: HashMap<String, u16> = forward.iter().map(|(k, v)| (v.clone(), *k)).collect();
        let codec = FontCodec::TwoByteMap { forward, reverse };

        let encoded = codec.encode("abcfi").unwrap();
        let decoded = codec.decode(&encoded);
        assert_eq!(decoded, "abcfi");
    }

    #[test]
    fn real_pdf_export() {
        let pdf = "/home/me/Projects/Gerda-hilft/Zeugnisse/ZeugnisJonathan.pdf";
        if !std::path::Path::new(pdf).exists() {
            eprintln!("Skipping: test PDF not found");
            return;
        }

        let target = std::env::temp_dir().join("notaperfecta_test_export.pdf");
        let corrections = vec![Correction {
            original: "Versetzung".to_string(),
            correction: "Vorsetzung".to_string(),
            position: 0,
        }];

        let result = correct_document(pdf, corrections, &target);
        assert!(result.is_ok(), "Export failed: {:?}", result.err());

        let output = std::process::Command::new("pdftotext")
            .args(["-nopgbrk", target.to_str().unwrap(), "-"])
            .output()
            .expect("pdftotext");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(
            text.contains("Vorsetzung"),
            "Correction not found in exported PDF. First 500 chars: {:?}",
            &text[..text.len().min(500)]
        );
        assert!(
            !text.contains("Versetzung"),
            "Original text should have been replaced"
        );
        eprintln!("✓ Exported PDF contains corrected text");

        std::fs::remove_file(&target).ok();
    }

    #[test]
    fn real_pdf_form_corruption_fix() {
        let pdf = "/home/me/Projects/Gerda-hilft/Zeugnisse/ZeugnisJonathan.pdf";
        if !std::path::Path::new(pdf).exists() {
            eprintln!("Skipping: test PDF not found");
            return;
        }

        let target = std::env::temp_dir().join("notaperfecta_test_form_corruption.pdf");
        let corrections = vec![Correction {
            original: "besuchtes schulspezif sches Prof l".to_string(),
            correction: "besuchtes schulspezifisches Profil".to_string(),
            position: 0,
        }];

        let result = correct_document(pdf, corrections, &target);
        assert!(result.is_ok(), "Export failed: {:?}", result.err());

        let output = std::process::Command::new("pdftotext")
            .args(["-nopgbrk", target.to_str().unwrap(), "-"])
            .output()
            .expect("pdftotext");
        let text = String::from_utf8_lossy(&output.stdout);
        eprintln!("Exported text around 'Profil': {:?}",
            text.find("Profil").or(text.find("Prof")).map(|i| {
                let start = i.saturating_sub(20);
                let end = (i + 40).min(text.len());
                &text[start..end]
            })
        );
        assert!(
            text.contains("schulspezifisches Profil"),
            "Form corruption correction not applied"
        );
        eprintln!("✓ Form corruption corrected in exported PDF");

        std::fs::remove_file(&target).ok();
    }
}
