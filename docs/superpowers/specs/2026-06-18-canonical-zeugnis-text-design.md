# Kanonischer Zeugnis-Text als KI-Input

**Datum:** 2026-06-18
**Status:** Genehmigt (Approach A), Umsetzung läuft

## Problem

Der KI-Pfad (`check_spelling_ai`) bekommt aktuell den Output von `pdftotext` **ohne** `-layout`
(`extract_text_from_pdf`). Bei dem mehrspaltigen sächsischen Zeugnisformular ist dieser Output
**komplett verwürfelt**: Spalten werden verschachtelt, Noten lösen sich von ihren Fächern,
benachbarte Werte verschmelzen (z. B. `Deutsch 4-` + `Englisch 4` → `44`). Auf so einem
instabilen, von PDF zu PDF unterschiedlichen Input lässt sich die KI **nicht** zuverlässig
abstimmen.

## Ziel

Da das Formular (Gymnasium, Halbjahresinformation Kl. 5–9 / Halbjahreszeugnis Kl. 10) **immer
dasselbe Layout** hat, überführen wir jedes Zeugnis in ein **immer identisches, kanonisches
Textgerüst**. Nur die Werte ändern sich; die Struktur und die Labels sind jedes Mal byte-gleich.
So sieht die KI immer dieselbe Form → sie kann zu 100 % darauf getrimmt werden.

**Scope (Option 1):** Der kanonische Text wird **nur intern** als KI-Input verwendet. Er ersetzt
den `pdftotext`-Output, der an die KI geht. Der PDF-Export bleibt **unverändert**.

## Kern-Invariante (damit der Export weiter funktioniert)

Der Bemerkungs-**Freitext** wird **wortgetreu** übernommen: jedes Wort bleibt exakt erhalten
(nur überflüssige Spalten-Whitespaces werden zu einem Leerzeichen zusammengefasst, leere Zeilen
entfallen). Wir normalisieren **Struktur und Labels**, **niemals** den Inhalt des Freitexts.

Folge: Jeder **echte** Tippfehler, den die KI im Freitext meldet, steht wortgleich auch im
Original-PDF — der bestehende In-Place-Export (`export.rs`) findet und ersetzt ihn weiterhin.
Reine `pdftotext`-Extraktionsartefakte (Buchstabe→Leerzeichen) entstehen mit `-layout` praktisch
nicht mehr und verschwinden damit als Fehlerquelle.

## Architektur (Approach A: gemeinsames `zeugnis`-Modul)

Heute existiert ein deterministischer Parser **privat** in `formcheck.rs`
(`struct ZeugnisInfo`, `fn parse`), genutzt nur für Formvorschriften. Er extrahiert eine
Teilmenge (Klasse, Name, Kopfnoten-Werte, Fachnoten-Werte ohne Fachnamen, Bemerkungen) aus
`pdftotext -layout`.

**Plan:**

1. Neues Modul `src-tauri/src/commands/zeugnis.rs` — die einzige Quelle der Wahrheit.
   - `struct Zeugnis` mit dem **vollständigen** Feldsatz (siehe unten).
   - `fn parse(layout_text: &str) -> Option<Zeugnis>` (umgezogen + erweitert aus `formcheck`).
   - `fn to_canonical_text(&self) -> String` — rendert das feste Skelett.
   - `fn from_pdf(path: &str) -> Result<Zeugnis, String>` — `pdftotext -layout` + `parse`.
2. `formcheck.rs` nutzt künftig `zeugnis::parse` statt der eigenen Kopie (keine
   Doppel-Logik, gleiche Werte für beide Pfade).
3. `ollama::check_spelling_ai` nimmt **`path` statt `text`** und baut den KI-Input via
   `zeugnis::from_pdf(path)?.to_canonical_text()`. Damit erhält die KI **immer** den
   kanonischen Text — unabhängig vom Frontend.
4. Frontend: beide Aufrufstellen (`useCorrections.ts`, `useFolderSession.ts`) rufen
   `check_spelling_ai` mit `{ path, ollamaUrl, modelOverride }` auf. Der jetzt überflüssige
   `extract_text_from_pdf`-Aufruf im KI-Fluss entfällt; das ungenutzte `text`-State wird nicht
   mehr befüllt.

`extract_text_from_pdf` (das Command) bleibt bestehen — es ist ein eigenständiges Command und
könnte anderweitig gebraucht werden; nur der KI-Fluss nutzt es nicht mehr.

## Erfasste Felder (`struct Zeugnis`)

- `schule: String`
- `dokumenttyp: String` (z. B. „Halbjahresinformation des Gymnasiums")
- `klasse: String` (z. B. „9/2")
- `schulhalbjahr: String` (z. B. „1. Schulhalbjahr")
- `schuljahr: String` (z. B. „2025/2026")
- `name: String` (voller „Vorname und Name")
- `kopfnoten: Vec<(String, String)>` — Betragen, Mitarbeit, Fleiß, Ordnung
- `faecher: Vec<(String, String)>` — Fachname → Note, in Lesereihenfolge (links vor rechts,
  Zeile für Zeile)
- `wahlpflicht: Vec<(String, String)>`
- `bemerkungen: Vec<String>` — Freitextzeilen, wortgetreu
- `datum: String`

## Kanonisches Format (Beispiel: Jonathan)

```
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

Datum: 6. Februar 2026
```

Ein Feld pro Zeile (deterministischer als zweispaltig). Leere Werte werden als `-` gerendert.
Fehlt ein ganzer Abschnitt, bleibt seine Überschrift mit leerem Inhalt erhalten, damit das
Skelett **immer** gleich ist.

## Parsing-Strategie (`-layout`)

- **Kopfzeilen-Felder:** je per Regex an ihrem Label verankert (Restzeile getrimmt).
- **Notenraster:** zweispaltig. Jede physische Zeile wird an der Spaltengrenze in links/rechts
  geteilt; je Hälfte ist die abschließende notenartige Marke (`\d[+-]?` oder `-`) die Note,
  der führende Text das Fachlabel. Mehrzeilige Zellen (z. B. „Gemeinschaftskunde/" … Note …
  „Rechtserziehung/Wirtschaft") werden über Label-Fragment-Akkumulation zusammengeführt.
- **Bemerkungen:** Block zwischen „Bemerkungen:" und „Datum:"; Zeilen trimmen, 2+ Leerzeichen
  → 1, leere Zeilen entfernen. Wörter bleiben unangetastet.

## Fehlerbehandlung

- `from_pdf` gibt bei fehlendem/fehlerhaftem `pdftotext` einen `Err(String)` zurück (deutsche
  Meldung), wie die bestehenden Commands.
- Lässt sich das Dokument nicht als Zeugnis parsen (`parse` → `None`), fällt `check_spelling_ai`
  auf den rohen `pdftotext`-Text zurück, damit nie weniger geprüft wird als heute.

## Tests (TDD)

- Fixture: der reale `pdftotext -layout`-Output des Referenz-Zeugnisses (Jonathan).
- `parse` extrahiert alle Felder korrekt (inkl. mehrzeiliger Gemeinschaftskunde-Zelle und
  leerer Felder wie „Technik/Computer -").
- `to_canonical_text` erzeugt exakt das oben gezeigte Skelett (Byte-Vergleich).
- Freitext-Invariante: jedes Wort aus den Original-Bemerkungen ist im kanonischen Text
  enthalten.
- Bestehende `formcheck`-Tests bleiben grün (gemeinsamer Parser).

## Bewusst NICHT im Scope

- Kein Template-basierter PDF-Export, keine Änderung an `export.rs`.
- Keine weiteren Schularten (nur Gymnasium Kl. 5–10).
- Keine sichtbare Anzeige des kanonischen Texts im UI.
