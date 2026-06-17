use super::pdf;
use crate::models::AiSuggestion;
use regex::Regex;

struct ZeugnisInfo {
    is_halbjahreszeugnis: bool,
    klassenstufe: u8,
    klasse_nummer: Option<u8>,
    vorname: String,
    kopfnoten: Vec<(String, String)>,
    fachnoten: Vec<String>,
    bemerkungen: String,
}

fn has_tendenz(note: &str) -> bool {
    let b = note.trim().as_bytes();
    b.len() == 2 && b[0].is_ascii_digit() && (b[1] == b'+' || b[1] == b'-')
}

fn grade_value(note: &str) -> Option<f32> {
    let b = note.trim().as_bytes();
    if b.is_empty() {
        return None;
    }
    let digit = b[0].wrapping_sub(b'0');
    if !(1..=6).contains(&digit) {
        return None;
    }
    let base = digit as f32;
    match b.get(1) {
        Some(b'+') if b.len() == 2 => Some(base - 0.33),
        Some(b'-') if b.len() == 2 => Some(base + 0.33),
        None => Some(base),
        _ => None,
    }
}

fn parse(text: &str) -> Option<ZeugnisInfo> {
    let is_halbjahreszeugnis = text.contains("Halbjahreszeugnis")
        || (text.contains("Jahreszeugnis") && !text.contains("Halbjahresinformation"));

    let klasse_re = Regex::new(r"Klasse:\s+(\d+)(?:/(\d+))?").ok()?;
    let caps = klasse_re.captures(text)?;
    let klassenstufe: u8 = caps.get(1)?.as_str().parse().ok()?;
    let klasse_nummer: Option<u8> = caps.get(2).and_then(|m| m.as_str().parse().ok());

    let vorname = Regex::new(r"(?m)Vorname und Name:\s+(\S+)")
        .ok()?
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    let kopfnoten = parse_kopfnoten(text);
    let fachnoten = parse_fachnoten(text);
    let bemerkungen = parse_bemerkungen(text);

    Some(ZeugnisInfo {
        is_halbjahreszeugnis,
        klassenstufe,
        klasse_nummer,
        vorname,
        kopfnoten,
        fachnoten,
        bemerkungen,
    })
}

fn parse_kopfnoten(text: &str) -> Vec<(String, String)> {
    let mut noten = Vec::new();

    for line in text.lines() {
        if line.contains("Betragen") && line.contains("Mitarbeit") {
            let re = Regex::new(r"Betragen\s{3,}(\S+).*Mitarbeit\s{3,}(\S+)").unwrap();
            if let Some(caps) = re.captures(line) {
                noten.push(("Betragen".into(), caps[1].to_string()));
                noten.push(("Mitarbeit".into(), caps[2].to_string()));
            }
        }
        if line.contains("Fleiß") && line.contains("Ordnung") {
            let re = Regex::new(r"Fleiß\s{3,}(\S+).*Ordnung\s{3,}(\S+)").unwrap();
            if let Some(caps) = re.captures(line) {
                noten.push(("Fleiß".into(), caps[1].to_string()));
                noten.push(("Ordnung".into(), caps[2].to_string()));
            }
        }
    }

    noten
}

fn parse_fachnoten(text: &str) -> Vec<String> {
    let start = match text.find("Leistungen in den einzelnen Fächern:") {
        Some(s) => s,
        None => return vec![],
    };
    let end = text.find("Bemerkungen:").unwrap_or(text.len());
    if start >= end {
        return vec![];
    }

    let section = &text[start..end];
    let re = Regex::new(r"\s{2,}(\d[+-]?)(?:\s|$)").unwrap();

    re.captures_iter(section)
        .filter_map(|c| {
            let g = c.get(1)?.as_str();
            let digit = g.as_bytes()[0].wrapping_sub(b'0');
            if (1..=6).contains(&digit) {
                Some(g.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn parse_bemerkungen(text: &str) -> String {
    let start = match text.find("Bemerkungen:") {
        Some(s) => s + "Bemerkungen:".len(),
        None => return String::new(),
    };
    let end = text.find("Datum:").unwrap_or(text.len());
    if start < end {
        text[start..end].trim().to_string()
    } else {
        String::new()
    }
}

fn expected_bemerkungen(info: &ZeugnisInfo) -> Vec<&'static str> {
    match info.klassenstufe {
        5 => vec![
            "Lernen lernen: erteilt",
            "individuelle Förderung Urban Dance und Informatik: erteilt",
        ],
        6 => vec![
            "individuelle Förderung 2. Fremdsprache und Informatik: erteilt",
        ],
        7 => vec!["individuelle Förderung Informatik: erteilt"],
        8 => match info.klasse_nummer {
            Some(1..=2) => vec!["individuelle Förderung Informatik: erteilt"],
            Some(3..=6) => vec!["individuelle Förderung Deutsch: erteilt"],
            _ => vec![],
        },
        9 => vec!["Zusatzstunde Informatik: erteilt"],
        _ => vec![],
    }
}

pub fn check(path: &str) -> Result<Vec<AiSuggestion>, String> {
    let text = pdf::extract_text_layout(path)?;
    let info = match parse(&text) {
        Some(i) => i,
        None => return Ok(vec![]),
    };

    let mut results: Vec<AiSuggestion> = Vec::new();
    let mut pos = 0usize;

    // 1: Kopfnoten mit Tendenz
    for (name, note) in &info.kopfnoten {
        if has_tendenz(note) {
            let digit = note.chars().next().unwrap();
            results.push(AiSuggestion {
                original: format!("{name} {note}"),
                correction: format!("{name} {digit}"),
                kind: "Formvorschrift".into(),
                position: pos,
                explanation: "Kopfnoten werden als Ziffern ohne Tendenz eingetragen (§25 Abs. 8 SOGYA).".into(),
            });
            pos += 1;
        }
    }

    // 2: Kl. 10 Fachnoten mit Tendenz
    if info.is_halbjahreszeugnis {
        for note in &info.fachnoten {
            if has_tendenz(note) {
                let digit = note.chars().next().unwrap();
                results.push(AiSuggestion {
                    original: note.clone(),
                    correction: digit.to_string(),
                    kind: "Formvorschrift".into(),
                    position: pos,
                    explanation: "Im Halbjahreszeugnis (Kl. 10) dürfen Fachnoten keine Tendenz (+/-) ausweisen.".into(),
                });
                pos += 1;
            }
        }
    }

    // 3: Versetzungsgefährdung
    let threshold: f32 = if info.is_halbjahreszeugnis { 5.0 } else { 4.33 };
    let has_bad_grade = info
        .fachnoten
        .iter()
        .any(|n| grade_value(n).is_some_and(|v| v >= threshold));

    let bm = info.bemerkungen.to_lowercase();
    let has_vermerk =
        (bm.contains("versetzung") && bm.contains("gefährdet")) || bm.contains("versetzungsgefährdet");

    if has_bad_grade && !has_vermerk {
        let schwelle = if info.is_halbjahreszeugnis { "5" } else { "4-" };
        results.push(AiSuggestion {
            original: "(fehlt)".into(),
            correction: format!("{} ist versetzungsgefährdet.", info.vorname),
            kind: "Formvorschrift".into(),
            position: pos,
            explanation: format!(
                "Bei Fachnoten ab {schwelle} muss der Versetzungsgefährdungs-Vermerk in den Bemerkungen stehen."
            ),
        });
        pos += 1;
    }

    // 4: Standard-Bemerkungen
    for expected in expected_bemerkungen(&info) {
        if !info.bemerkungen.contains(expected) {
            results.push(AiSuggestion {
                original: "(fehlt)".into(),
                correction: expected.to_string(),
                kind: "Formvorschrift".into(),
                position: pos,
                explanation: format!(
                    "Standard-Bemerkung für Klassenstufe {} der Gerda-Taro-Schule fehlt.",
                    info.klassenstufe
                ),
            });
            pos += 1;
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tendenz_detection() {
        assert!(has_tendenz("3+"));
        assert!(has_tendenz("4-"));
        assert!(!has_tendenz("3"));
        assert!(!has_tendenz("keine"));
        assert!(!has_tendenz("-"));
    }

    #[test]
    fn grade_ordering() {
        assert!(grade_value("4-").unwrap() > grade_value("4").unwrap());
        assert!(grade_value("4").unwrap() > grade_value("4+").unwrap());
        assert!(grade_value("5").unwrap() > grade_value("4-").unwrap());
        assert_eq!(grade_value("-"), None);
        assert_eq!(grade_value("keine"), None);
    }

    #[test]
    fn versetzung_threshold_kl59() {
        let threshold: f32 = 4.33;
        assert!(grade_value("4-").unwrap() >= threshold);
        assert!(grade_value("5+").unwrap() >= threshold);
        assert!(grade_value("5").unwrap() >= threshold);
        assert!(grade_value("4").unwrap() < threshold);
        assert!(grade_value("3-").unwrap() < threshold);
    }

    #[test]
    fn versetzung_threshold_kl10() {
        let threshold: f32 = 5.0;
        assert!(grade_value("5").unwrap() >= threshold);
        assert!(grade_value("6").unwrap() >= threshold);
        assert!(grade_value("4").unwrap() < threshold);
    }

    #[test]
    fn parses_kopfnoten_from_layout() {
        let text = "Betragen                    3               Mitarbeit                     4\n\
                     Fleiß                       4               Ordnung                       4";
        let noten = parse_kopfnoten(text);
        assert_eq!(noten.len(), 4);
        assert_eq!(noten[0], ("Betragen".into(), "3".into()));
        assert_eq!(noten[1], ("Mitarbeit".into(), "4".into()));
    }

    #[test]
    fn parses_fachnoten_from_layout() {
        let text = "Leistungen in den einzelnen Fächern:\n\
                     Deutsch                    4-               Mathematik                    4\n\
                     Englisch                   4                Biologie                     4+\n\
                     2. Fremdsprache (ab Klassenstufe 6)\n\
                     Bemerkungen:";
        let noten = parse_fachnoten(text);
        assert_eq!(noten, vec!["4-", "4", "4", "4+"]);
    }

    #[test]
    fn parses_bemerkungen() {
        let text = "Bemerkungen:   Fehltage entschuldigt: 3   unentschuldigt: 0\n\
                     Jonathans Versetzung ist gefährdet.\n\
                     Zusatzstunde Informatik: erteilt\n\n\
                     Datum:   6. Februar 2026";
        let bem = parse_bemerkungen(text);
        assert!(bem.contains("Versetzung"));
        assert!(bem.contains("Zusatzstunde"));
        assert!(!bem.contains("Datum"));
    }
}
