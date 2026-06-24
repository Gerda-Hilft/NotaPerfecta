//! Deterministic parser for the fixed Saxon Gymnasium report form
//! (Halbjahresinformation Kl. 5–9 / Halbjahreszeugnis Kl. 10).
//!
//! Turns the `pdftotext -layout` output into a `Zeugnis` struct and renders an
//! always-identical canonical text. The canonical text is the input the AI is
//! tuned on: same skeleton every time, only the values differ. The Bemerkungen
//! free text is carried over word-for-word so the in-place PDF export can still
//! locate any real typo the AI reports.

use super::pdf;

/// Char column that separates the left and right cell of the two-column grid.
/// Left grades sit at col ~59–60, right labels start at col ~76, so any split
/// in between is safe.
const COL_SPLIT: usize = 70;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Zeugnis {
    pub schule: String,
    pub dokumenttyp: String,
    pub klasse: String,
    pub schulhalbjahr: String,
    pub schuljahr: String,
    pub name: String,
    pub kopfnoten: Vec<(String, String)>,
    pub faecher: Vec<(String, String)>,
    pub wahlpflicht: Vec<(String, String)>,
    pub bemerkungen: Vec<String>,
    pub datum: String,
}

impl Zeugnis {
    /// Parse the `pdftotext -layout` output. Returns `None` if the text does not
    /// look like one of these report forms (no "Klasse:" anchor).
    pub fn parse(layout: &str) -> Option<Zeugnis> {
        if !layout.contains("Klasse:") {
            return None;
        }
        let lines: Vec<&str> = layout.lines().collect();
        let find = |needle: &str| lines.iter().position(|l| l.contains(needle));

        let mut z = Zeugnis::default();

        if let Some(idx) = find("Name der Schule:") {
            z.schule = after_label(lines[idx], "Name der Schule:");
        }

        z.dokumenttyp = lines
            .iter()
            .map(|l| l.trim())
            .find(|t| {
                t.starts_with("Halbjahresinformation")
                    || t.starts_with("Halbjahreszeugnis")
                    || t.starts_with("Jahreszeugnis")
            })
            .unwrap_or("")
            .to_string();

        if let Some(idx) = find("Klasse:") {
            let segs = segments(lines[idx]);
            if let Some(pos) = segs.iter().position(|(_, t)| t == "Klasse:") {
                if let Some((_, v)) = segs.get(pos + 1) {
                    z.klasse = v.clone();
                }
            }
            for (_, t) in &segs {
                if t.contains("Schulhalbjahr") {
                    z.schulhalbjahr = t.clone();
                }
                if is_schuljahr(t) {
                    z.schuljahr = t.clone();
                }
            }
        }

        if let Some(idx) = find("Vorname und Name:") {
            z.name = after_label(lines[idx], "Vorname und Name:");
        }
        if let Some(idx) = find("Datum:") {
            z.datum = after_label(lines[idx], "Datum:");
        }

        let leistungen = find("Leistungen in den einzelnen");
        let wahlpflicht = find("Wahlpflichtbereich");
        let bemerkungen = find("Bemerkungen:");
        let datum_idx = find("Datum:");
        let name_idx = find("Vorname und Name:");

        if let (Some(n), Some(l)) = (name_idx, leistungen) {
            let rows: Vec<_> = lines[n + 1..l].iter().map(|ln| row_cells(ln)).collect();
            z.kopfnoten = collect_pairs(&rows);
        }

        if let Some(l) = leistungen {
            let end = wahlpflicht.or(bemerkungen).unwrap_or(lines.len());
            if l + 1 < end {
                let rows: Vec<_> = lines[l + 1..end].iter().map(|ln| row_cells(ln)).collect();
                z.faecher = collect_pairs(&rows);
            }
        }

        if let Some(w) = wahlpflicht {
            let end = bemerkungen.unwrap_or(lines.len());
            if w + 1 < end {
                let rows: Vec<_> = lines[w + 1..end].iter().map(|ln| row_cells(ln)).collect();
                z.wahlpflicht = collect_pairs(&rows);
            }
        }

        if let Some(b) = bemerkungen {
            let end = datum_idx.unwrap_or(lines.len());
            let mut out = Vec::new();
            for (offset, ln) in lines[b..end].iter().enumerate() {
                let content = if offset == 0 {
                    after_label(ln, "Bemerkungen:")
                } else {
                    (*ln).to_string()
                };
                let norm = content.split_whitespace().collect::<Vec<_>>().join(" ");
                if !norm.is_empty() {
                    out.push(norm);
                }
            }
            z.bemerkungen = out;
        }

        Some(z)
    }

    /// Extract + parse straight from a PDF file.
    pub fn from_pdf(path: &str) -> Result<Zeugnis, String> {
        let layout = pdf::extract_text_layout(path)?;
        Zeugnis::parse(&layout).ok_or_else(|| "Kein Zeugnis-Formular erkannt".to_string())
    }

    /// Render the always-identical canonical skeleton.
    pub fn to_canonical_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("=== {} ===\n", self.dokumenttyp.to_uppercase()));
        out.push_str(&format!("Schule: {}\n", self.schule));
        out.push_str(&format!("Klasse: {}\n", self.klasse));
        out.push_str(&format!("Schulhalbjahr: {} {}\n", self.schulhalbjahr, self.schuljahr));
        out.push_str(&format!("Vorname und Name: {}\n", self.name));

        out.push_str("\nKOPFNOTEN\n");
        for (k, v) in &self.kopfnoten {
            out.push_str(&format!("{k}: {v}\n"));
        }

        out.push_str("\nLEISTUNGEN IN DEN EINZELNEN FÄCHERN\n");
        for (k, v) in &self.faecher {
            out.push_str(&format!("{k}: {v}\n"));
        }

        out.push_str("\nWAHLPFLICHTBEREICH\n");
        for (k, v) in &self.wahlpflicht {
            out.push_str(&format!("{k}: {v}\n"));
        }

        out.push_str("\nBEMERKUNGEN\n");
        for line in &self.bemerkungen {
            out.push_str(line);
            out.push('\n');
        }

        out.push_str(&format!("\nDatum: {}", self.datum));
        out
    }

    // ---- Accessors used by the Formvorschrift checks (formcheck.rs) ----

    pub fn is_halbjahreszeugnis(&self) -> bool {
        let d = &self.dokumenttyp;
        d.contains("Halbjahreszeugnis")
            || (d.contains("Jahreszeugnis") && !d.contains("Halbjahresinformation"))
    }

    pub fn klassenstufe(&self) -> u8 {
        self.klasse
            .split('/')
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    pub fn klasse_nummer(&self) -> Option<u8> {
        self.klasse.split('/').nth(1).and_then(|s| s.trim().parse().ok())
    }

    pub fn vorname(&self) -> String {
        self.name.split_whitespace().next().unwrap_or("").to_string()
    }

    pub fn fachnoten(&self) -> Vec<String> {
        self.faecher.iter().map(|(_, note)| note.clone()).collect()
    }

    pub fn bemerkungen_text(&self) -> String {
        self.bemerkungen.join("\n")
    }
}

/// A parsed grid cell: its label text and an optional grade token.
type Cell = (String, Option<String>);

/// Split a line into segments separated by runs of 2+ spaces, keeping single
/// spaces inside a segment. Returns each segment with its starting char column.
fn segments(line: &str) -> Vec<(usize, String)> {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut segs = Vec::new();
    let mut i = 0;
    while i < n {
        while i < n && chars[i] == ' ' {
            i += 1;
        }
        if i >= n {
            break;
        }
        let start = i;
        let mut text = String::new();
        while i < n {
            if chars[i] == ' ' {
                if i + 1 < n && chars[i + 1] == ' ' {
                    break; // run of 2+ spaces = boundary
                }
                text.push(' ');
                i += 1;
            } else {
                text.push(chars[i]);
                i += 1;
            }
        }
        segs.push((start, text.trim_end().to_string()));
    }
    segs
}

/// True for a single grade token: `-`, a digit 1–6, or a digit with tendency.
fn is_grade_token(s: &str) -> bool {
    let s = s.trim();
    if s == "-" {
        return true;
    }
    let b = s.as_bytes();
    match b.len() {
        1 => (b'1'..=b'6').contains(&b[0]),
        2 => (b'1'..=b'6').contains(&b[0]) && (b[1] == b'+' || b[1] == b'-'),
        _ => false,
    }
}

/// Split one side's segments into (label, optional grade).
fn split_cell(segs: &[(usize, String)]) -> Cell {
    if segs.is_empty() {
        return (String::new(), None);
    }
    let last = &segs[segs.len() - 1].1;
    if is_grade_token(last) {
        let label = segs[..segs.len() - 1]
            .iter()
            .map(|(_, t)| t.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        (label, Some(last.trim().to_string()))
    } else {
        let label = segs.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>().join(" ");
        (label, None)
    }
}

/// Split a physical grid line into its left and right cell at `COL_SPLIT`.
fn row_cells(line: &str) -> (Cell, Cell) {
    let segs = segments(line);
    let left: Vec<_> = segs.iter().filter(|(c, _)| *c < COL_SPLIT).cloned().collect();
    let right: Vec<_> = segs.iter().filter(|(c, _)| *c >= COL_SPLIT).cloned().collect();
    (split_cell(&left), split_cell(&right))
}

/// Join wrapped label fragments. No separator after a `/` (or `-`), else a space.
fn join_label(parts: &[String]) -> String {
    let mut out = String::new();
    for p in parts {
        if out.is_empty() || out.ends_with('/') || out.ends_with('-') {
            out.push_str(p);
        } else {
            out.push(' ');
            out.push_str(p);
        }
    }
    out
}

fn is_grade_row(c: &(Cell, Cell)) -> bool {
    c.0 .1.is_some() || c.1 .1.is_some()
}

/// For a grade whose inline label is empty (a multi-line cell), assemble its
/// label from the orphan label-only lines directly above and below.
fn wrapped_label(rows: &[(Cell, Cell)], i: usize, left: bool) -> String {
    let side = |c: &(Cell, Cell)| -> String {
        if left { c.0 .0.clone() } else { c.1 .0.clone() }
    };

    let mut above: Vec<String> = Vec::new();
    let mut j = i;
    while j > 0 {
        j -= 1;
        if is_grade_row(&rows[j]) {
            break;
        }
        let lab = side(&rows[j]);
        if lab.is_empty() {
            break;
        }
        above.push(lab);
    }
    above.reverse();

    let mut below: Vec<String> = Vec::new();
    let mut k = i + 1;
    while k < rows.len() {
        if is_grade_row(&rows[k]) {
            break;
        }
        let lab = side(&rows[k]);
        if lab.is_empty() {
            break;
        }
        below.push(lab);
        k += 1;
    }

    let mut parts = above;
    parts.extend(below);
    join_label(&parts)
}

/// Walk grid rows top-to-bottom, emitting (label, grade) pairs in reading order
/// (left cell before right cell), skipping captions and empty (`-`) slots.
fn collect_pairs(rows: &[(Cell, Cell)]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for i in 0..rows.len() {
        let (l, r) = &rows[i];
        if !is_grade_row(&rows[i]) {
            continue;
        }
        if let Some(g) = &l.1 {
            let label = if l.0.is_empty() {
                wrapped_label(rows, i, true)
            } else {
                l.0.clone()
            };
            push_pair(&mut out, label, g.clone());
        }
        if let Some(g) = &r.1 {
            let label = if r.0.is_empty() {
                wrapped_label(rows, i, false)
            } else {
                r.0.clone()
            };
            push_pair(&mut out, label, g.clone());
        }
    }
    out
}

fn push_pair(out: &mut Vec<(String, String)>, label: String, grade: String) {
    let label = label.trim().to_string();
    if label.is_empty() || label == "-" {
        return;
    }
    out.push((label, grade));
}

fn after_label(line: &str, label: &str) -> String {
    match line.split_once(label) {
        Some((_, rest)) => rest.trim().to_string(),
        None => String::new(),
    }
}

fn is_schuljahr(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 9
        && b[4] == b'/'
        && b[..4].iter().all(|c| c.is_ascii_digit())
        && b[5..].iter().all(|c| c.is_ascii_digit())
}

/// Fixed template strings that appear verbatim on every Gymnasium report form.
/// If one is missing but a near-identical (corrupted) variant exists, the form
/// text was damaged — e.g. a letter turned into a space during PDF export.
const FORM_LABELS: &[&str] = &[
    "Name der Schule",
    "des Gymnasiums",
    "Vorname und Name",
    "Leistungen in den einzelnen Fächern",
    "Wahlpflichtbereich",
    "besuchtes schulspezifisches Profil",
    "Fehltage entschuldigt",
    "unentschuldigt",
    "Bemerkungen",
    "Klassenlehrer(in)",
    "Zur Kenntnis genommen",
    "Notenerläuterung",
];

/// Levenshtein edit distance between two char slices.
fn levenshtein(a: &[char], b: &[char]) -> usize {
    let n = b.len();
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Find the best near-match for `label` in the text (a corrupted variant).
/// Returns the matched fragment if its edit distance is small but nonzero.
fn best_fuzzy_match(chars: &[char], label: &str) -> Option<String> {
    let e: Vec<char> = label.chars().collect();
    let elen = e.len();
    if elen < 8 {
        return None; // too short to disambiguate reliably
    }
    let threshold = if elen < 20 { 1 } else { 2 };
    let min_wl = elen.saturating_sub(2).max(4);
    let mut best: Option<(usize, String)> = None;

    for wl in min_wl..=elen + 2 {
        if wl > chars.len() {
            break;
        }
        for start in 0..=chars.len() - wl {
            let w = &chars[start..start + wl];
            // Anchor: corruption keeps the word ends intact, so require a shared
            // 4-char prefix or suffix. This kills spurious matches.
            let pref = w[..4] == e[..4];
            let suf = w[w.len() - 4..] == e[elen - 4..];
            if !(pref || suf) {
                continue;
            }
            let d = levenshtein(w, &e);
            if (1..=threshold).contains(&d) && best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                best = Some((d, w.iter().collect::<String>()));
            }
        }
    }
    best.map(|(_, s)| s.trim().to_string())
}

/// Detect damaged fixed form text. Returns `(corrupted_fragment, expected_text)`
/// pairs for every template string that is missing verbatim but present in a
/// near-identical, corrupted form.
pub fn find_form_corruptions(layout: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = layout.chars().collect();
    let mut out: Vec<(String, String)> = Vec::new();
    for &label in FORM_LABELS {
        if layout.contains(label) {
            continue;
        }
        if let Some(found) = best_fuzzy_match(&chars, label) {
            if !out.iter().any(|(o, _)| o == &found) {
                out.push((found, label.to_string()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("testdata/zeugnis_jonathan_layout.txt");

    const EXPECTED_CANONICAL: &str = "\
=== HALBJAHRESINFORMATION DES GYMNASIUMS ===
Schule: Gerda-Taro-Schule, Gymnasium der Stadt Leipzig
Klasse: 9/2
Schulhalbjahr: 1. Schulhalbjahr 2025/2026
Vorname und Name: Jonathan Soppa

KOPFNOTEN
Betragen: 3
Mitarbeit: 4
Fleiß: 4
Ordnung: 4

LEISTUNGEN IN DEN EINZELNEN FÄCHERN
Deutsch: 4-
Mathematik: 4
Englisch: 4
Biologie: 4+
Französisch: 5
Chemie: 5
Kunst: 3
Physik: 3
Musik: 2
Sport: 2
Geschichte: 3
Ethik: 3-
Gemeinschaftskunde/Rechtserziehung/Wirtschaft: 4-
Technik/Computer: -
Geographie: 4
Informatik: 4

WAHLPFLICHTBEREICH
Informatik und Gesellschaft: 5

BEMERKUNGEN
Fehltage entschuldigt: 3 unentschuldigt: 0
Jonathans Versetzung in Klasse 10 ist gefährdet.
Zusatzstunde Informatik: erteilt

Datum: 6. Februar 2026";

    fn parsed() -> Zeugnis {
        Zeugnis::parse(FIXTURE).expect("fixture should parse")
    }

    #[test]
    fn parses_header_fields() {
        let z = parsed();
        assert_eq!(z.schule, "Gerda-Taro-Schule, Gymnasium der Stadt Leipzig");
        assert_eq!(z.dokumenttyp, "Halbjahresinformation des Gymnasiums");
        assert_eq!(z.klasse, "9/2");
        assert_eq!(z.schulhalbjahr, "1. Schulhalbjahr");
        assert_eq!(z.schuljahr, "2025/2026");
        assert_eq!(z.name, "Jonathan Soppa");
        assert_eq!(z.datum, "6. Februar 2026");
    }

    #[test]
    fn parses_kopfnoten() {
        let z = parsed();
        assert_eq!(
            z.kopfnoten,
            vec![
                ("Betragen".to_string(), "3".to_string()),
                ("Mitarbeit".to_string(), "4".to_string()),
                ("Fleiß".to_string(), "4".to_string()),
                ("Ordnung".to_string(), "4".to_string()),
            ]
        );
    }

    #[test]
    fn parses_faecher_in_reading_order() {
        let z = parsed();
        let expected: Vec<(String, String)> = [
            ("Deutsch", "4-"),
            ("Mathematik", "4"),
            ("Englisch", "4"),
            ("Biologie", "4+"),
            ("Französisch", "5"),
            ("Chemie", "5"),
            ("Kunst", "3"),
            ("Physik", "3"),
            ("Musik", "2"),
            ("Sport", "2"),
            ("Geschichte", "3"),
            ("Ethik", "3-"),
            ("Gemeinschaftskunde/Rechtserziehung/Wirtschaft", "4-"),
            ("Technik/Computer", "-"),
            ("Geographie", "4"),
            ("Informatik", "4"),
        ]
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
        assert_eq!(z.faecher, expected);
    }

    #[test]
    fn drops_captions_and_empty_wahlpflicht_slot() {
        let z = parsed();
        // The "2. Fremdsprache (ab Klassenstufe 6)" caption must not become a subject.
        assert!(!z.faecher.iter().any(|(name, _)| name.contains("Fremdsprache")));
        // Only the filled Wahlpflicht entry survives; the empty right slot ("-") is dropped.
        assert_eq!(
            z.wahlpflicht,
            vec![("Informatik und Gesellschaft".to_string(), "5".to_string())]
        );
    }

    #[test]
    fn parses_bemerkungen_verbatim_words() {
        let z = parsed();
        assert_eq!(
            z.bemerkungen,
            vec![
                "Fehltage entschuldigt: 3 unentschuldigt: 0".to_string(),
                "Jonathans Versetzung in Klasse 10 ist gefährdet.".to_string(),
                "Zusatzstunde Informatik: erteilt".to_string(),
            ]
        );
    }

    #[test]
    fn renders_exact_canonical_text() {
        assert_eq!(parsed().to_canonical_text(), EXPECTED_CANONICAL);
    }

    #[test]
    fn freetext_words_survive_into_canonical() {
        // Every word of the original Bemerkungen free text must appear in the
        // canonical text, so the export can still find any typo the AI reports.
        let canonical = parsed().to_canonical_text();
        for word in "Jonathans Versetzung in Klasse 10 ist gefährdet.".split_whitespace() {
            assert!(canonical.contains(word), "missing word: {word}");
        }
    }

    #[test]
    fn accessors_for_formcheck() {
        let z = parsed();
        assert_eq!(z.klassenstufe(), 9);
        assert_eq!(z.klasse_nummer(), Some(2));
        assert!(!z.is_halbjahreszeugnis());
        assert_eq!(z.vorname(), "Jonathan");
        assert_eq!(
            z.fachnoten(),
            vec!["4-", "4", "4", "4+", "5", "5", "3", "3", "2", "2", "3", "3-", "4-", "-", "4", "4"]
        );
    }

    #[test]
    fn non_zeugnis_returns_none() {
        assert!(Zeugnis::parse("irgendein anderer Text ohne Formular").is_none());
    }

    fn cs(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein(&cs("abc"), &cs("abc")), 0);
        assert_eq!(levenshtein(&cs("kitten"), &cs("sitting")), 3);
        // a letter turned into a space = one substitution
        assert_eq!(
            levenshtein(&cs("Wahlpflichtbereich"), &cs("Wahlpf ichtbereich")),
            1
        );
    }

    #[test]
    fn clean_form_has_no_corruptions() {
        assert_eq!(find_form_corruptions(FIXTURE), Vec::<(String, String)>::new());
    }

    #[test]
    fn detects_corrupted_heading() {
        // "Wahlpflichtbereich" with the 'l' replaced by a space, as the user reported.
        let corrupted = FIXTURE.replace("Wahlpflichtbereich", "Wahlpf ichtbereich");
        let found = find_form_corruptions(&corrupted);
        assert_eq!(found.len(), 1, "exactly one corruption expected, got {found:?}");
        assert_eq!(found[0].0, "Wahlpf ichtbereich");
        assert_eq!(found[0].1, "Wahlpflichtbereich");
    }

    #[test]
    fn detects_corrupted_section_title() {
        // A space inserted into "Bemerkungen".
        let corrupted = FIXTURE.replace("Bemerkungen", "Bemer kungen");
        let found = find_form_corruptions(&corrupted);
        assert!(
            found.iter().any(|(_, corr)| corr == "Bemerkungen"),
            "expected Bemerkungen corruption, got {found:?}"
        );
    }

    #[test]
    fn detects_typo_in_unentschuldigt() {
        // The reported case: "unentschuldigt" with the first 'n' dropped. The AI
        // whitelists this standard term, so it must be caught deterministically.
        let corrupted = FIXTURE.replace("unentschuldigt", "unetschuldigt");
        let found = find_form_corruptions(&corrupted);
        assert!(
            found
                .iter()
                .any(|(o, c)| o == "unetschuldigt" && c == "unentschuldigt"),
            "expected unentschuldigt typo, got {found:?}"
        );
    }

    #[test]
    fn does_not_flag_clean_label_with_corruption_elsewhere() {
        // Corrupting one label must not produce false positives for the others.
        let corrupted = FIXTURE.replace("Notenerläuterung", "Notenerl uterung");
        let found = find_form_corruptions(&corrupted);
        assert_eq!(found.len(), 1, "got {found:?}");
        assert_eq!(found[0].1, "Notenerläuterung");
    }
}
