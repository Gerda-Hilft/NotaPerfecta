import { invoke } from "@tauri-apps/api/core";
import { useMemo, useState } from "react";
import type { ExportKorrektur, KorrekturVorschlag, PipelineModus } from "../types/corrections";

type BackendSuggestion = {
  original: string;
  correction: string;
  type: "Rechtschreibung" | "Grammatik" | "Zeichensetzung";
  position: number;
  explanation: string;
  source: "KI" | "Wörterbuch";
};

function dedupe(suggestions: BackendSuggestion[]): KorrekturVorschlag[] {
  const map = new Map<string, KorrekturVorschlag>();
  for (const s of suggestions) {
    const key = `${s.original}|${s.correction}|${s.position}`;
    if (!map.has(key)) {
      map.set(key, { ...s, id: key, status: "offen" });
    }
  }
  return [...map.values()];
}

export function useCorrections() {
  const [text, setText] = useState("");
  const [path, setPath] = useState("");
  const [loadingKi, setLoadingKi] = useState(false);
  const [loadingWb, setLoadingWb] = useState(false);
  const [error, setError] = useState("");
  const [vorschlaege, setVorschlaege] = useState<KorrekturVorschlag[]>([]);

  const status = useMemo(() => {
    const angenommen = vorschlaege.filter((v) => v.status === "angenommen").length;
    const abgelehnt = vorschlaege.filter((v) => v.status === "abgelehnt").length;
    return { gesamt: vorschlaege.length, angenommen, abgelehnt };
  }, [vorschlaege]);

  async function analysiere(pdfPath: string, modus: PipelineModus) {
    setError("");
    setPath(pdfPath);
    const extracted = await invoke<string>("extract_text_from_pdf", { path: pdfPath });
    setText(extracted);

    const jobs: Promise<BackendSuggestion[]>[] = [];
    if (modus !== "woerterbuch") {
      setLoadingKi(true);
      jobs.push(
        invoke<BackendSuggestion[]>("check_spelling_ai", { text: extracted })
          .catch(() => {
            setError("KI nicht erreichbar - nur Wörterbuch-Modus verfügbar");
            return [];
          })
          .finally(() => setLoadingKi(false)),
      );
    }

    if (modus !== "ki") {
      setLoadingWb(true);
      jobs.push(
        invoke<BackendSuggestion[]>("check_spelling_dictionary", { text: extracted }).finally(() =>
          setLoadingWb(false),
        ),
      );
    }

    const data = (await Promise.all(jobs)).flat();
    setVorschlaege(dedupe(data));
  }

  function markiere(id: string, statusWert: "angenommen" | "abgelehnt") {
    setVorschlaege((prev) => prev.map((v) => (v.id === id ? { ...v, status: statusWert } : v)));
  }

  function bulk(statusWert: "angenommen" | "abgelehnt") {
    setVorschlaege((prev) => prev.map((v) => ({ ...v, status: statusWert })));
  }

  async function exportiere() {
    const accepted: ExportKorrektur[] = vorschlaege
      .filter((v) => v.status === "angenommen")
      .map(({ original, correction, position }) => ({ original, correction, position }));

    return invoke<string>("export_corrected_pdf", {
      originalPath: path,
      acceptedCorrections: accepted,
    });
  }

  return {
    text,
    loadingKi,
    loadingWb,
    error,
    vorschlaege,
    status,
    analysiere,
    markiere,
    bulk,
    exportiere,
  };
}
