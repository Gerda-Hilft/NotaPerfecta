import { open } from "@tauri-apps/plugin-dialog";
import { useState, type DragEvent } from "react";

interface Props {
  onFileSelected: (path: string) => void;
  onFolderSelected: (path: string) => void;
  onFehler: (nachricht: string) => void;
}

function laeuftInTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function DropZone({ onFileSelected, onFolderSelected, onFehler }: Props) {
  const [dragAktiv, setDragAktiv] = useState(false);

  async function waehleDatei() {
    if (!laeuftInTauri()) {
      onFehler("Bitte als Desktop-App mit 'pnpm tauri dev' starten.");
      return;
    }

    const selected = await open({
      multiple: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (typeof selected === "string") {
      onFileSelected(selected);
    }
  }

  async function waehleOrdner() {
    if (!laeuftInTauri()) {
      onFehler("Bitte als Desktop-App mit 'pnpm tauri dev' starten.");
      return;
    }

    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (typeof selected === "string") {
      onFolderSelected(selected);
    }
  }

  function onDrop(e: DragEvent<HTMLDivElement>) {
    e.preventDefault();
    setDragAktiv(false);

    if (!laeuftInTauri()) {
      onFehler("Drag-and-drop ist nur in der Desktop-App verfuegbar.");
      return;
    }

    const file = e.dataTransfer.files?.[0] as File & { path?: string };
    const pfad = file?.path;
    if (pfad && pfad.toLowerCase().endsWith(".pdf")) {
      onFileSelected(pfad);
      return;
    }
    onFehler("Bitte eine PDF-Datei ablegen.");
  }

  return (
    <div
      className={`dropzone ${dragAktiv ? "drag-aktiv" : ""}`}
      onDragOver={(e) => {
        e.preventDefault();
        setDragAktiv(true);
      }}
      onDragLeave={() => setDragAktiv(false)}
      onDrop={onDrop}
    >
      <div className="dropzone-content">
        <span className="dropzone-text">Zeugnis hier ablegen oder:</span>
        <div className="dropzone-buttons">
          <button className="btn btn-primary btn-sm" onClick={waehleDatei}>
            Einzelnes PDF
          </button>
          <button className="btn btn-outline btn-sm" onClick={waehleOrdner}>
            Ganzer Ordner
          </button>
        </div>
      </div>
    </div>
  );
}
