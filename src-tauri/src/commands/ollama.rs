use super::pdf;
use super::zeugnis::Zeugnis;
use crate::models::{AiSuggestion, RawAiSuggestion};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

fn prompt(text: &str) -> String {
    format!(
        r#"Du bist ein spezialisierter Lektor für sächsische Schulzeugnisse (Halbjahresinformationen Kl. 5–9, Halbjahreszeugnisse Kl. 10).

Der folgende Text wurde aus einem Zeugnis-PDF extrahiert. Beim PDF-Export kann es vorkommen, dass einzelne Buchstaben durch Leerzeichen ersetzt werden (z. B. „un ntschuldigt" statt „unentschuldigt"). Prüfe NUR den Freitext in den Bemerkungen auf Rechtschreib-, Grammatik- und Zeichensetzungsfehler — einschließlich solcher Export-Artefakte.

Formvorschriften (Notentendenzen, Versetzungsgefährdung, Standard-Bemerkungen) werden separat geprüft — melde dazu NICHTS.

NIEMALS als Fehler melden:
• Fächernamen, Kopfnoten-Bezeichnungen, Formular-Überschriften und Labels
• Eigennamen (Personen, Schulen, Orte)
• Geschlechtergerechte Formen (Lehrerinnen, Schülerin, Lehrer:innen usw.)
• Zahlen, Daten, Klassenstufen, Schuljahre, Noten
• Notenerläuterung
• Standardformulierungen wenn sie KORREKT geschrieben sind: „Versetzung … gefährdet", „Fehltage entschuldigt:", „unentschuldigt", „erteilt"
• Stilistische Alternativen — nur eindeutige Fehler

IMMER als Fehler melden:
• Eindeutige Tippfehler (fehlende, vertauschte oder zusätzliche Buchstaben) in JEDEM Wort, auch wenn es einer Standardformulierung ähnelt. Beispiel: „unetschuldigt" statt „unentschuldigt" ist ein Rechtschreibfehler.
• Export-Artefakte: Wörter, bei denen ein Buchstabe durch ein Leerzeichen ersetzt wurde (z. B. „un ntschuldigt", „entsc huldigt"). Das „original" ist das betroffene Wortfragment inkl. des falsch platzierten Leerzeichens; „korrektur" ist das korrekte zusammengeschriebene Wort; „typ" ist „Rechtschreibung".
• Das Feld „original" muss das kleinste fehlerhafte Textstück enthalten (einzelnes Wort oder Fragment, nicht den gesamten Satz oder das Label).

Bei Unsicherheit: NICHT melden.

Antworte NUR mit einem JSON-Array. Kein Text davor oder danach. Bei keinen Fehlern: []
[
  {{
    "original": "fehlerhaftes Wort oder Phrase",
    "korrektur": "korrigierte Version",
    "typ": "Rechtschreibung" | "Grammatik" | "Zeichensetzung",
    "erklaerung": "kurze Begründung auf Deutsch"
  }}
]

Text:
{}"#,
        text
    )
}

fn extract_json_array(input: &str) -> Option<String> {
    let re = Regex::new(r"\[[\s\S]*\]").ok()?;
    re.find(input).map(|m| m.as_str().to_string())
}

const GENDER_SUFFIXES: [&str; 2] = ["innen", "in"];
const GENDER_SEPARATORS: [char; 5] = ['*', ':', '_', '/', '-'];

const ZEUGNIS_IGNORE: &[&str] = &[
    "Halbjahresinformation",
    "Halbjahreszeugnis",
    "Jahreszeugnis",
    "Gymnasium",
    "Gymnasiums",
    "Schulhalbjahr",
    "Klassenstufe",
    "Fremdsprache",
    "Wahlpflichtbereich",
    "Notenerläuterung",
    "Klassenlehrer",
    "Klassenlehrer(in)",
    "Klassenlehrerin",
    "Betragen",
    "Mitarbeit",
    "Fleiß",
    "Ordnung",
    "Deutsch",
    "Mathematik",
    "Englisch",
    "Biologie",
    "Französisch",
    "Chemie",
    "Kunst",
    "Physik",
    "Musik",
    "Sport",
    "Geschichte",
    "Ethik",
    "Religion",
    "Gemeinschaftskunde",
    "Rechtserziehung",
    "Wirtschaft",
    "Geographie",
    "Informatik",
    "Technik",
    "Computer",
    "sehr gut",
    "gut",
    "befriedigend",
    "ausreichend",
    "mangelhaft",
    "ungenügend",
    "entschuldigt",
    "unentschuldigt",
    "Fehltage",
    "Bemerkungen",
    "Versetzung",
    "gefährdet",
    "versetzungsgefährdet",
    "erteilt",
    "nicht erteilt",
    "teilgenommen",
    "befreit",
    "nicht bewertet",
    "keine Benotung",
    "Zusatzstunde",
    "Freistaat",
    "Sachsen",
    "schulspezifisches",
    "Profil",
    "Kenntnis",
    "genommen",
    "Lernen lernen",
    "individuelle Förderung",
    "Urban Dance",
    "Laufbahn",
    "M.I.T.",
    "Klassenkonferenz",
    "Schulleiterin",
];

fn is_zeugnis_term(original: &str) -> bool {
    let trimmed = original.trim();
    ZEUGNIS_IGNORE
        .iter()
        .any(|term| trimmed.eq_ignore_ascii_case(term))
}

fn is_grade(s: &str) -> bool {
    let t = s.trim();
    matches!(
        t,
        "1" | "1+"
            | "1-"
            | "2"
            | "2+"
            | "2-"
            | "3"
            | "3+"
            | "3-"
            | "4"
            | "4+"
            | "4-"
            | "5"
            | "5+"
            | "5-"
            | "6+"
            | "6"
            | "-"
    )
}

/// Returns the gender-neutral base of `word` if it ends in a feminine or
/// gender-inclusive suffix (e.g. "Lehrerinnen", "Lehrer:in", "LehrerInnen"),
/// otherwise `None`.
fn strip_gender_suffix(word: &str) -> Option<String> {
    for suffix in GENDER_SUFFIXES {
        if let Some(stripped) = word.strip_suffix(suffix) {
            let base = stripped.trim_end_matches(GENDER_SEPARATORS);
            if base.chars().count() >= 3 {
                return Some(base.to_string());
            }
        }
    }
    None
}

/// True if the suggested change does nothing but strip a feminine or
/// gender-inclusive suffix from `original` (e.g. "Lehrerinnen" -> "Lehrer").
/// Used as a safety net in case the model ignores the prompt instructions.
fn is_gender_removal(original: &str, correction: &str) -> bool {
    let o = original.trim().to_lowercase();
    let c = correction.trim().to_lowercase();
    match strip_gender_suffix(&o) {
        Some(base) => base == c,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_feminine_and_inclusive_forms_being_stripped() {
        assert!(is_gender_removal("Lehrerinnen", "Lehrer"));
        assert!(is_gender_removal("Lehrerin", "Lehrer"));
        assert!(is_gender_removal("Schülerinnen", "Schüler"));
        assert!(is_gender_removal("Lehrer:innen", "Lehrer"));
        assert!(is_gender_removal("Lehrer*innen", "Lehrer"));
        assert!(is_gender_removal("Lehrer_innen", "Lehrer"));
        assert!(is_gender_removal("LehrerInnen", "Lehrer"));
    }

    #[test]
    fn keeps_unrelated_or_purely_grammatical_changes() {
        // both forms stay gendered (singular/plural agreement) -> not a removal
        assert!(!is_gender_removal("Lehrerin", "Lehrerinnen"));
        // unrelated singular/plural correction, no gendering involved
        assert!(!is_gender_removal("Termine", "Termin"));
        // genuine spelling fix
        assert!(!is_gender_removal("Klasenarbeit", "Klassenarbeit"));
        // no change
        assert!(!is_gender_removal("Mathematik", "Mathematik"));
    }
}

pub async fn list_ollama_models(ollama_url: String) -> Result<Vec<String>, String> {
    let base = ollama_url.trim_end_matches('/');
    let client = Client::new();

    let tags: Value = client
        .get(format!("{base}/api/tags"))
        .send()
        .await
        .map_err(|e| format!("Verbindung zu Ollama fehlgeschlagen: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Antwort von Ollama ungueltig: {e}"))?;

    let models = tags
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

pub async fn check_spelling_ai(
    path: String,
    ollama_url: String,
    model_override: String,
) -> Result<Vec<AiSuggestion>, String> {
    // The AI is tuned on the canonical text: same skeleton for every Zeugnis,
    // only the values differ. Fall back to raw extraction if the PDF is not a
    // recognized report form, so we never check less than before.
    let text = match Zeugnis::from_pdf(&path) {
        Ok(z) => z.to_canonical_text(),
        Err(_) => pdf::extract_text_from_pdf(path).await?,
    };

    let base = ollama_url.trim_end_matches('/');
    let client = Client::new();

    let model = if model_override.is_empty() {
        let tags: Value = client
            .get(format!("{base}/api/tags"))
            .send()
            .await
            .map_err(|e| format!("Verbindung zu Ollama fehlgeschlagen: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Antwort von Ollama ungueltig: {e}"))?;

        tags.get("models")
            .and_then(|m| m.as_array())
            .and_then(|arr| {
                arr.iter()
                    .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                    .find(|name| name.starts_with("llama3"))
                    .or_else(|| {
                        arr.iter()
                            .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                            .next()
                    })
            })
            .ok_or_else(|| "Kein Ollama-Modell gefunden".to_string())?
            .to_string()
    } else {
        model_override
    };

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt(&text),
        "stream": false
    });

    let response: Value = client
        .post(format!("{base}/api/generate"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anfrage an Ollama fehlgeschlagen: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Antwort von KI ungueltig: {e}"))?;

    let raw_response = response
        .get("response")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "KI-Antwort ohne Inhalt".to_string())?;

    let parsed: Vec<RawAiSuggestion> = match extract_json_array(raw_response) {
        Some(arr_text) => serde_json::from_str(&arr_text)
            .map_err(|e| format!("KI-JSON konnte nicht verarbeitet werden: {e}"))?,
        None => Vec::new(),
    };

    let suggestions = parsed
        .into_iter()
        .filter(|s| {
            !is_gender_removal(&s.original, &s.correction)
                && !is_zeugnis_term(&s.original)
                && !is_grade(&s.original)
        })
        .enumerate()
        .map(|(i, s)| AiSuggestion {
            original: s.original,
            correction: s.correction,
            kind: s.kind,
            position: i,
            explanation: s.explanation,
        })
        .collect();

    Ok(suggestions)
}
