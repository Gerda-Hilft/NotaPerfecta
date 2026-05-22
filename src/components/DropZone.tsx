import { open } from "@tauri-apps/plugin-dialog";
import { useState, type DragEvent } from "react";

interface Props {
  onFileSelected: (path: string) => void;
  onFehler: (nachricht: string) => void;
}

function laeuftInTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function DropZone({ onFileSelected, onFehler }: Props) {
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

  function onDrop(e: DragEvent<HTMLButtonElement>) {
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
    <button
      className={`dropzone ${dragAktiv ? "drag-aktiv" : ""}`}
      onClick={waehleDatei}
      onDragOver={(e) => {
        e.preventDefault();
        setDragAktiv(true);
      }}
      onDragLeave={() => setDragAktiv(false)}
      onDrop={onDrop}
    >
      Zeugnis hier ablegen oder klicken
    </button>
  );
}
