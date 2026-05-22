import { useState } from "react";
import "./App.css";
import { CorrectionList } from "./components/CorrectionList";
import { DropZone } from "./components/DropZone";
import { ExportButton } from "./components/ExportButton";
import { PipelineToggle } from "./components/PipelineToggle";
import { useCorrections } from "./hooks/useCorrections";
import type { PipelineModus } from "./types/corrections";

function App() {
  const [modus, setModus] = useState<PipelineModus>("beides");
  const [meldung, setMeldung] = useState("");
  const { text, loadingKi, loadingWb, error, vorschlaege, status, analysiere, markiere, bulk, exportiere } =
    useCorrections();

  async function onFile(path: string) {
    setMeldung("");
    try {
      await analysiere(path, modus);
    } catch (e) {
      setMeldung(`Fehler beim Verarbeiten: ${String(e)}`);
    }
  }

  function onFehler(nachricht: string) {
    setMeldung(nachricht);
  }

  async function onExport() {
    try {
      const out = await exportiere();
      setMeldung(`Export abgeschlossen: ${out}`);
    } catch (e) {
      setMeldung(`Export fehlgeschlagen: ${String(e)}`);
    }
  }

  return (
    <main className="app">
      <h1>NotaPerfecta - Zeugnisprüfung</h1>
      <PipelineToggle value={modus} onChange={setModus} />
      <DropZone onFileSelected={onFile} onFehler={onFehler} />

      <div className="status">
        <span>{status.gesamt} Fehler gefunden</span>
        <span>{status.angenommen} angenommen</span>
        <span>{status.abgelehnt} abgelehnt</span>
      </div>

      {(loadingKi || loadingWb) && (
        <div className="loading">
          {loadingKi && <span>KI analysiert… </span>}
          {loadingWb && <span>Wörterbuch prüft…</span>}
        </div>
      )}

      {(error || meldung) && <p className="meldung">{error || meldung}</p>}

      <section className="split">
        <article className="panel">
          <h2>Originaltext</h2>
          <pre>{text || "Noch kein Zeugnis geladen."}</pre>
        </article>
        <article className="panel">
          <h2>Korrekturen</h2>
          <div className="toolbar">
            <button onClick={() => bulk("angenommen")}>Alle annehmen</button>
            <button onClick={() => bulk("abgelehnt")}>Alle ablehnen</button>
            <ExportButton disabled={!vorschlaege.length} onExport={onExport} />
          </div>
          <CorrectionList
            suggestions={vorschlaege}
            onAccept={(id) => markiere(id, "angenommen")}
            onReject={(id) => markiere(id, "abgelehnt")}
          />
        </article>
      </section>
    </main>
  );
}

export default App;
