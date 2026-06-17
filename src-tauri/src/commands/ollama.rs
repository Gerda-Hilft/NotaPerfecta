use crate::models::{AiSuggestion, RawAiSuggestion};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

fn prompt(text: &str) -> String {
    format!(
        "Du bist ein Lektor für deutsche Schulzeugnisse. Analysiere den folgenden Text ausschließlich auf eindeutige Rechtschreib-, Grammatik- und Zeichensetzungsfehler.\n\nNICHT als Fehler melden, niemals ändern:\n- Geschlechtergerechte und weibliche Formen (z. B. \"Lehrerinnen\", \"Schülerin\", \"Erzieher:innen\", \"Kolleg*innen\", \"MitarbeiterInnen\"). Diese sind korrekt, auch wenn die männliche Form gebräuchlicher ist, und dürfen NIEMALS in eine generische männliche Form geändert oder entfernt werden.\n- Eigennamen von Personen, Schulen, Orten, Fächern oder Marken, auch wenn sie ungewöhnlich erscheinen.\n- Zahlen, Daten, Noten und Maßeinheiten.\n- Übliche Zeugnis-Standardformulierungen (z. B. \"stets\", \"in der Regel\", \"zuverlässig\", \"mit großem Interesse\").\n- Stilistische Alternativen. Melde nur, wenn etwas eindeutig falsch ist.\n\nWenn du unsicher bist, ob etwas ein Fehler ist, melde es NICHT.\n\nAntworte NUR mit einem JSON-Array. Kein erklärender Text davor oder danach. Format:\n[\n  {{\n    \"original\": \"fehlerhaftes Wort oder Phrase\",\n    \"korrektur\": \"korrigierte Version\",\n    \"typ\": \"Rechtschreibung\" | \"Grammatik\" | \"Zeichensetzung\",\n    \"erklaerung\": \"kurze Begründung auf Deutsch\"\n  }}\n]\n\nText:\n{}",
        text
    )
}

fn extract_json_array(input: &str) -> Option<String> {
    let re = Regex::new(r"\[[\s\S]*\]").ok()?;
    re.find(input).map(|m| m.as_str().to_string())
}

const GENDER_SUFFIXES: [&str; 2] = ["innen", "in"];
const GENDER_SEPARATORS: [char; 5] = ['*', ':', '_', '/', '-'];

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
    text: String,
    ollama_url: String,
    model_override: String,
) -> Result<Vec<AiSuggestion>, String> {
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
        .filter(|s| !is_gender_removal(&s.original, &s.correction))
        .enumerate()
        .map(|(i, s)| AiSuggestion {
            original: s.original,
            correction: s.correction,
            kind: s.kind,
            position: i,
            explanation: s.explanation,
            source: "KI".to_string(),
        })
        .collect();

    Ok(suggestions)
}
