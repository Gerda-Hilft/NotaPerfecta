use super::pdf;
use super::zeugnis::{self, Zeugnis};
use crate::models::AiSuggestion;

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

fn expected_bemerkungen(z: &Zeugnis) -> Vec<&'static str> {
    match z.klassenstufe() {
        5 => vec![
            "Lernen lernen: erteilt",
            "individuelle Förderung Urban Dance und Informatik: erteilt",
        ],
        6 => vec!["individuelle Förderung 2. Fremdsprache und Informatik: erteilt"],
        7 => vec!["individuelle Förderung Informatik: erteilt"],
        8 => match z.klasse_nummer() {
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

    let mut results: Vec<AiSuggestion> = Vec::new();
    let mut pos = 0usize;

    // 0: Beschädigter Formulartext (z. B. "Wahlpf ichtbereich" statt
    //    "Wahlpflichtbereich"). Läuft unabhängig vom Parsen, damit eine
    //    Beschädigung auch dann gemeldet wird, wenn sie das Parsen stört.
    for (original, correction) in zeugnis::find_form_corruptions(&text) {
        results.push(AiSuggestion {
            original,
            correction,
            kind: "Rechtschreibung".into(),
            position: pos,
            explanation: "Fester Formulartext ist beschädigt (vermutlich Export-Artefakt) und sollte exakt der amtlichen Vorlage entsprechen.".into(),
        });
        pos += 1;
    }

    let z = match Zeugnis::parse(&text) {
        Some(z) => z,
        None => return Ok(results),
    };

    // 1: Kopfnoten mit Tendenz
    for (name, note) in &z.kopfnoten {
        if has_tendenz(note) {
            let digit = note.chars().next().unwrap();
            results.push(AiSuggestion {
                original: format!("{name} {note}"),
                correction: format!("{name} {digit}"),
                kind: "Formvorschrift".into(),
                position: pos,
                explanation:
                    "Kopfnoten werden als Ziffern ohne Tendenz eingetragen (§25 Abs. 8 SOGYA)."
                        .into(),
            });
            pos += 1;
        }
    }

    // 2: Kl. 10 Fachnoten mit Tendenz
    if z.is_halbjahreszeugnis() {
        for note in &z.fachnoten() {
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
    let threshold: f32 = if z.is_halbjahreszeugnis() { 5.0 } else { 4.33 };
    let has_bad_grade = z
        .fachnoten()
        .iter()
        .any(|n| grade_value(n).is_some_and(|v| v >= threshold));

    let bm = z.bemerkungen_text().to_lowercase();
    let has_vermerk = (bm.contains("versetzung") && bm.contains("gefährdet"))
        || bm.contains("versetzungsgefährdet");

    if has_bad_grade && !has_vermerk {
        let schwelle = if z.is_halbjahreszeugnis() { "5" } else { "4-" };
        results.push(AiSuggestion {
            original: "(fehlt)".into(),
            correction: format!("{} ist versetzungsgefährdet.", z.vorname()),
            kind: "Formvorschrift".into(),
            position: pos,
            explanation: format!(
                "Bei Fachnoten ab {schwelle} muss der Versetzungsgefährdungs-Vermerk in den Bemerkungen stehen."
            ),
        });
        pos += 1;
    }

    // 4: Standard-Bemerkungen
    let bemerkungen_text = z.bemerkungen_text();
    for expected in expected_bemerkungen(&z) {
        if !bemerkungen_text.contains(expected) {
            results.push(AiSuggestion {
                original: "(fehlt)".into(),
                correction: expected.to_string(),
                kind: "Formvorschrift".into(),
                position: pos,
                explanation: format!(
                    "Standard-Bemerkung für Klassenstufe {} der Schule fehlt.",
                    z.klassenstufe()
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
}
