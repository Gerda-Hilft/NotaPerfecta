import { useState } from "react";
import "./App.css";
import { Background } from "./components/Background";
import { CorrectionList } from "./components/CorrectionList";
import { DropZone } from "./components/DropZone";
import { ExportButton } from "./components/ExportButton";
import { FolderSidebar } from "./components/FolderSidebar";
import { SettingsDialog } from "./components/SettingsDialog";
import { useCorrections } from "./hooks/useCorrections";
import { useFolderSession } from "./hooks/useFolderSession";
import { useSettings } from "./hooks/useSettings";
import { PdfViewer } from "./components/PdfViewer";

function App() {
  const {
    settings,
    update: updateSettings,
    reset: resetSettings,
  } = useSettings();
  const [meldung, setMeldung] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);

  const single = useCorrections();
  const folder = useFolderSession();

  const inFolderMode = folder.ordner !== null;

  async function onFile(path: string) {
    setMeldung("");
    folder.schliesseOrdner();
    try {
      await single.analysiere(path, {
        ollamaUrl: settings.ollamaUrl,
        ollamaModel: settings.ollamaModel,
      });
    } catch (e) {
      setMeldung(`Fehler beim Verarbeiten: ${String(e)}`);
    }
  }

  async function onFolder(path: string) {
    setMeldung("");
    try {
      await folder.oeffneOrdner(path);
    } catch (e) {
      setMeldung(String(e));
    }
  }

  function onFehler(nachricht: string) {
    setMeldung(nachricht);
  }

  async function onExportSingle() {
    try {
      const out = await single.exportiere();
      setMeldung(`Export abgeschlossen: ${out}`);
    } catch (e) {
      setMeldung(`Export fehlgeschlagen: ${String(e)}`);
    }
  }

  async function onSelectDatei(pfad: string) {
    await folder.waehleDatei(pfad, {
      ollamaUrl: settings.ollamaUrl,
      ollamaModel: settings.ollamaModel,
    });
  }

  const current = folder.aktuell;

  return (
    <div className="app-shell">
      <Background />
      <main className="app">
        <div className="app-header">
          <h1>NotaPerfecta</h1>
          <div className="app-header-actions">
            <button
              className="btn btn-outline btn-sm"
              onClick={() => setSettingsOpen(true)}
            >
              Einstellungen
            </button>
          </div>
        </div>

        {!inFolderMode && (
          <DropZone
            onFileSelected={onFile}
            onFolderSelected={onFolder}
            onFehler={onFehler}
          />
        )}

        {(single.error || meldung) && (
          <p className="meldung">{single.error || meldung}</p>
        )}

        {inFolderMode ? (
          <div className="folder-layout">
            <FolderSidebar
              ordnerName={folder.ordner!}
              dateien={folder.dateien}
              aktuellerPfad={folder.aktuellerPfad}
              exportStatus={folder.exportStatus}
              onSelect={onSelectDatei}
              onExportAll={folder.exportiereAlle}
              onClose={folder.schliesseOrdner}
            />
            <div className="folder-main">
              {current === null ? (
                <div className="folder-empty panel">
                  <p>
                    Wähle ein Zeugnis aus der Liste, um Korrekturen zu prüfen.
                  </p>
                </div>
              ) : current.phase === "wird-analysiert" ? (
                <div className="folder-empty panel">
                  <div className="loading">
                    <span>Wird analysiert…</span>
                  </div>
                </div>
              ) : current.phase === "fehler" ? (
                <div className="panel">
                  <p className="meldung">{current.error}</p>
                </div>
              ) : (
                <>
                  <article className="panel folder-panel">
                    <h2>Originaltext — {current.filename}</h2>
                    <PdfViewer
                      pdfPath={current.path}
                      suggestions={current.vorschlaege}
                    />
                  </article>
                  <article className="panel folder-panel">
                    <h2>
                      Korrekturen
                      {current.vorschlaege.length > 0 && (
                        <span
                          className="badge"
                          style={{ marginLeft: "0.5rem" }}
                        >
                          {current.vorschlaege.length}
                        </span>
                      )}
                    </h2>
                    {current.vorschlaege.length > 0 ? (
                      <>
                        <div className="toolbar">
                          <button
                            className="btn btn-outline btn-sm"
                            onClick={() => folder.bulk("angenommen")}
                          >
                            Alle annehmen
                          </button>
                          <button
                            className="btn btn-outline btn-sm"
                            onClick={() => folder.bulk("abgelehnt")}
                          >
                            Alle ablehnen
                          </button>
                        </div>
                        <CorrectionList
                          suggestions={current.vorschlaege}
                          onAccept={(id) => folder.markiere(id, "angenommen")}
                          onReject={(id) => folder.markiere(id, "abgelehnt")}
                        />
                      </>
                    ) : (
                      <p style={{ color: "var(--color-success)" }}>
                        Keine Fehler gefunden.
                      </p>
                    )}
                  </article>
                </>
              )}
            </div>
          </div>
        ) : (
          <>
            <div className="status">
              <span className="badge">
                {single.status.gesamt} Fehler gefunden
              </span>
              <span className="badge badge-success">
                {single.status.angenommen} angenommen
              </span>
              <span className="badge badge-muted">
                {single.status.abgelehnt} abgelehnt
              </span>
            </div>

            {single.loadingKi && (
              <div className="loading">
                <span>KI analysiert…</span>
              </div>
            )}

            <section className="split">
              <article className="panel">
                <h2>Originaltext</h2>
                {single.path ? (
                  <PdfViewer
                    pdfPath={single.path}
                    suggestions={single.vorschlaege}
                  />
                ) : (
                  <p
                    style={{
                      color: "var(--color-muted, #888)",
                      padding: "1rem",
                    }}
                  >
                    Noch kein Zeugnis geladen.
                  </p>
                )}
              </article>
              <article className="panel">
                <h2>Korrekturen</h2>
                <div className="toolbar">
                  <button
                    className="btn btn-outline btn-sm"
                    onClick={() => single.bulk("angenommen")}
                  >
                    Alle annehmen
                  </button>
                  <button
                    className="btn btn-outline btn-sm"
                    onClick={() => single.bulk("abgelehnt")}
                  >
                    Alle ablehnen
                  </button>
                  <ExportButton
                    disabled={!single.vorschlaege.length}
                    onExport={onExportSingle}
                  />
                </div>
                <CorrectionList
                  suggestions={single.vorschlaege}
                  onAccept={(id) => single.markiere(id, "angenommen")}
                  onReject={(id) => single.markiere(id, "abgelehnt")}
                />
              </article>
            </section>
          </>
        )}

        <SettingsDialog
          open={settingsOpen}
          settings={settings}
          onUpdate={updateSettings}
          onReset={resetSettings}
          onClose={() => setSettingsOpen(false)}
        />
      </main>
    </div>
  );
}

export default App;
