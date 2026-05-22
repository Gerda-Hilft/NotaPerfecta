use crate::models::SpellingError;
use strsim::levenshtein;

const GERMAN_WORDS: &[&str] = &[
    "der", "die", "das", "und", "ist", "im", "in", "mit", "zu", "den", "dem", "ein",
    "eine", "einer", "einem", "eines", "schueler", "schuelerin", "leistung", "leistungen",
    "verhalten", "arbeitsverhalten", "sozialverhalten", "unterricht", "deutsch", "mathematik",
    "englisch", "sehr", "gut", "befriedigend", "ausreichend", "entwickelt", "zeigt", "seine",
    "ihre", "aufgaben", "arbeitet", "konzentriert", "zuverlaessig", "regelmaessig", "teil",
    "nimmt", "freundlich", "respektvoll", "klasse", "schule", "zeugnis", "halbjahr", "jahr",
    "verbessert", "noch", "kann", "sollte", "hat", "haben", "wurde", "werden", "am", "an",
    "vom", "zur", "zum", "und", "oder", "auch", "nicht", "stets", "meistens", "oft",
    "komposita", "lernbereitschaft", "arbeitsweise", "teamfaehigkeit", "kommunikationsfaehigkeit",
];

fn normalize(word: &str) -> String {
    word.to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
}

pub async fn check_spelling_dictionary(text: String) -> Result<Vec<SpellingError>, String> {
    let mut results = Vec::new();

    for (idx, token) in text.split_whitespace().enumerate() {
        let cleaned = token
            .trim_matches(|c: char| !c.is_alphabetic() && c != 'ä' && c != 'ö' && c != 'ü' && c != 'ß');

        if cleaned.len() < 3 {
            continue;
        }

        let norm = normalize(cleaned);
        if GERMAN_WORDS.contains(&norm.as_str()) {
            continue;
        }

        let mut best = "";
        let mut best_dist = usize::MAX;
        for candidate in GERMAN_WORDS {
            let dist = levenshtein(&norm, candidate);
            if dist < best_dist {
                best_dist = dist;
                best = candidate;
            }
        }

        if best_dist <= 2 {
            results.push(SpellingError {
                original: cleaned.to_string(),
                correction: best.to_string(),
                kind: "Rechtschreibung".to_string(),
                position: idx,
                explanation: "Wort ist nicht im Wörterbuch, ähnlichster Treffer vorgeschlagen.".to_string(),
                source: "Wörterbuch".to_string(),
            });
        }
    }

    Ok(results)
}
