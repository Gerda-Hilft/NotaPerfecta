use crate::models::{AiSuggestion, RawAiSuggestion};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

fn prompt(text: &str) -> String {
    format!(
        "Du bist ein Lektor fuer deutsche Schulzeugnisse. Analysiere den folgenden Text auf Rechtschreib- und Grammatikfehler.\n\nAntworte NUR mit einem JSON-Array. Kein erklaerender Text davor oder danach. Format:\n[\n  {{\n    \"original\": \"fehlerhaftes Wort oder Phrase\",\n    \"korrektur\": \"korrigierte Version\",\n    \"typ\": \"Rechtschreibung\" | \"Grammatik\" | \"Zeichensetzung\",\n    \"erklaerung\": \"kurze Begruendung auf Deutsch\"\n  }}\n]\n\nText:\n{}",
        text
    )
}

fn extract_json_array(input: &str) -> Option<String> {
    let re = Regex::new(r"\[[\s\S]*\]").ok()?;
    re.find(input).map(|m| m.as_str().to_string())
}

pub async fn check_spelling_ai(text: String) -> Result<Vec<AiSuggestion>, String> {
    let client = Client::new();

    let tags: Value = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map_err(|_| "KI nicht erreichbar - nur Woerterbuch-Modus verfuegbar".to_string())?
        .json()
        .await
        .map_err(|e| format!("Antwort von Ollama ungueltig: {e}"))?;

    let model = tags
        .get("models")
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
        .ok_or_else(|| "Kein Ollama-Modell gefunden".to_string())?;

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt(&text),
        "stream": false,
        "format": "json"
    });

    let response: Value = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .send()
        .await
        .map_err(|_| "KI nicht erreichbar - nur Woerterbuch-Modus verfuegbar".to_string())?
        .json()
        .await
        .map_err(|e| format!("Antwort von KI ungueltig: {e}"))?;

    let raw_response = response
        .get("response")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "KI-Antwort ohne Inhalt".to_string())?;

    let arr_text = extract_json_array(raw_response).unwrap_or_else(|| raw_response.to_string());

    let parsed: Vec<RawAiSuggestion> = serde_json::from_str(&arr_text)
        .or_else(|_| serde_json::from_str(raw_response))
        .map_err(|e| format!("KI-JSON konnte nicht verarbeitet werden: {e}"))?;

    let suggestions = parsed
        .into_iter()
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
