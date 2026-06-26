# NotaPerfecta

KI-gestützte Korrekturhilfe für sächsische Schulzeugnisse. Kombiniert regelbasierte Formvorschriften-Prüfung mit lokalem KI-Rechtschreibcheck via [Ollama](https://ollama.com).

---

## Funktionen

### PDF-Analyse
- **Drag & Drop** einer einzelnen Zeugnis-PDF oder eines ganzen Ordners mit mehreren Zeugnissen
- **PDF-Viewer** mit Live-Hervorhebungen, die erkannte Fehler direkt im Dokument markieren

### Zweistufige Prüfung

| Stufe | Was wird geprüft |
|---|---|
| **Formvorschriften** (sofort, offline) | Kopfnoten mit Notentendenzen (+/-), Fachnoten-Tendenzen im Halbjahreszeugnis Kl. 10, fehlender Versetzungsgefährdungs-Vermerk, fehlende klassenstufenspezifische Standard-Bemerkungen |
| **KI-Rechtschreibung** (via Ollama) | Rechtschreibung, Grammatik, Zeichensetzung und PDF-Export-Artefakte (z. B. `un ntschuldigt`) ausschließlich im Freitext — Formularbeschriftungen, Fächernamen, Eigennamen, Noten und geschlechtergerechte Formen werden nie beanstandet |

### Korrektur-Workflow
- Korrekturen erscheinen in einem Seitenpanel neben der PDF
- Jeden Vorschlag einzeln **annehmen** oder **ablehnen**
- **Alle annehmen / Alle ablehnen** für schnelle Durchläufe
- **Export** — schreibt eine korrigierte Kopie der PDF mit allen angenommenen Änderungen; das Original bleibt unverändert

### Ordner-Modus
- Ordner öffnen, um alle PDFs in der Seitenleiste aufzulisten
- Zeugnisse einzeln analysieren oder alle auf einmal exportieren
- Export-Status pro Datei in der Seitenleiste

### Einstellungen
- Ollama-Server-URL konfigurieren (lokal oder remote)
- Spezifisches Modell auswählen oder automatisch erkennen lassen (bevorzugt `llama3`)
- Einstellungen werden sitzungsübergreifend gespeichert

---

## Installation

Fertige Installer sind auf der [Releases-Seite](../../releases) verfügbar.

| Plattform | Datei |
|---|---|
| macOS | `.dmg` |
| Linux | `.appimage` |
| Windows | `.exe` |

### macOS
1. Die `.dmg`-Datei herunterladen
2. Öffnen und **NotaPerfecta** in den Programme-Ordner ziehen
3. Beim ersten Start zeigt macOS ggf. eine Sicherheitswarnung — unter **Systemeinstellungen → Datenschutz & Sicherheit** auf **Trotzdem öffnen** klicken

### Linux
1. Die `.appimage`-Datei herunterladen
2. Ausführbar machen: `chmod +x NotaPerfecta_*.appimage`
3. Starten: `./NotaPerfecta_*.appimage`

### Windows
1. Den `.exe`-Installer herunterladen
2. Ausführen und dem Setup-Assistenten folgen
3. Windows Defender SmartScreen kann beim ersten Start warnen — auf **Weitere Informationen → Trotzdem ausführen** klicken

---

## Ollama einrichten

NotaPerfecta benötigt eine laufende [Ollama](https://ollama.com)-Instanz mit mindestens einem heruntergeladenen Modell.

```bash
# Ollama installieren (siehe https://ollama.com für betriebssystemspezifische Anweisungen)
ollama pull gemma4:e2b         # empfohlenes Modell (~7.3 GB)
```

Standardmäßig verbindet sich die App mit `http://127.0.0.1:11434`. Die URL kann in den **Einstellungen** geändert werden, falls Ollama auf einem anderen Gerät oder Port läuft.

**Remote-Ollama-Server:** Die URL auf die Adresse des Servers setzen (z. B. `http://192.168.1.10:11434`). Der extrahierte PDF-Text wird zur Analyse an Ollama gesendet — das sollte bei Servern außerhalb des lokalen Netzwerks bedacht werden.

---

## Entwicklung

### Voraussetzungen

| Tool | Version |
|---|---|
| [Rust](https://rustup.rs) | stable (1.77+) |
| [Bun](https://bun.sh) | 1.x |
| [Tauri CLI](https://tauri.app/reference/cli/) | v2 (wird über Bun installiert) |

**Linux** benötigt zusätzlich die Tauri-Systemabhängigkeiten:

```bash
# Debian / Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Arch / CachyOS
sudo pacman -S webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg
```

**macOS** — die Xcode Command Line Tools reichen aus: `xcode-select --install`

**Windows** — die [Microsoft C++ Build Tools](https://aka.ms/buildtools) und WebView2 installieren (ist ab Windows 11 / Edge enthalten).

### Abhängigkeiten installieren

```bash
bun install
```

### Entwicklungsmodus starten

```bash
bun run tauri dev
```

Das Frontend lädt bei Dateiänderungen automatisch neu. Das Rust-Backend wird bei Änderungen neu kompiliert.

### Frontend-Tests ausführen

```bash
bun run test
```

### Produktions-Installer bauen

```bash
bun run tauri build
```

Die Ausgabe liegt in `src-tauri/target/release/bundle/`.

---

## Projektstruktur

```
NotaPerfecta/
├── src/                        # React-Frontend (TypeScript)
│   ├── components/             # UI-Komponenten
│   │   ├── PdfViewer.tsx       # PDF-Canvas + Highlight-Overlays
│   │   ├── CorrectionList.tsx  # Annehmen/Ablehnen-Liste
│   │   ├── FolderSidebar.tsx   # Dateiliste im Ordner-Modus
│   │   ├── DropZone.tsx        # Datei-/Ordner-Dropziel
│   │   ├── ExportButton.tsx
│   │   └── SettingsDialog.tsx
│   ├── hooks/
│   │   ├── useCorrections.ts   # Einzeldatei-Analyse & Export-State
│   │   ├── useFolderSession.ts # Ordner-Modus-State
│   │   ├── useSettings.ts      # Persistierte Einstellungen
│   │   └── useNotifications.tsx
│   └── lib/
│       └── pdfHighlight.ts     # Textposition → Canvas-Overlay-Logik
└── src-tauri/                  # Rust-Backend (Tauri)
    └── src/
        └── commands/
            ├── ollama.rs       # KI-Rechtschreibcheck via Ollama-API
            ├── formcheck.rs    # Regelbasierte Formvorschriften-Prüfung
            ├── pdf.rs          # PDF-Textextraktion
            ├── export.rs       # Korrigierte PDF-Generierung
            └── zeugnis.rs      # Zeugnis-Parser (kanonischer Textaufbau)
```

---

## Empfohlene IDEs

**VS Code** mit den Extensions:
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

**WebStorm** — volle TypeScript/React-Unterstützung out of the box; Rust-Plugin: [IntelliJ Rust](https://plugins.jetbrains.com/plugin/8182-rust)

**Zed** — mit den Extensions `rust-analyzer` und `tauri` aus dem Extension-Hub

---

## Lizenz

Siehe [LICENSE](LICENSE).
