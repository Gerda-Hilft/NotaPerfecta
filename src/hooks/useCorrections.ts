import { invoke } from "@tauri-apps/api/core";
import { useMemo, useState } from "react";
import type { ExportKorrektur, KorrekturVorschlag } from "../types/corrections";

type BackendSuggestion = {
  original: string;
  correction: string;
  type: "Rechtschreibung" | "Grammatik" | "Zeichensetzung" | "Formvorschrift";
  position: number;
  explanation: string;
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

interface CorrectionOptions {
  ollamaUrl: string;
  ollamaModel: string;
}

export function useCorrections() {
  const [path, setPath] = useState("");
  const [loadingKi, setLoadingKi] = useState(false);
  const [error, setError] = useState("");
  const [vorschlaege, setVorschlaege] = useState<KorrekturVorschlag[]>([]);

  const status = useMemo(() => {
    const angenommen = vorschlaege.filter((v) => v.status === "angenommen").length;
    const abgelehnt = vorschlaege.filter((v) => v.status === "abgelehnt").length;
    return { gesamt: vorschlaege.length, angenommen, abgelehnt };
  }, [vorschlaege]);

  async function analysiere(pdfPath: string, opts?: CorrectionOptions) {
    setError("");
    setPath(pdfPath);

    const ollamaUrl = opts?.ollamaUrl || "http://127.0.0.1:11434";
    const modelOverride = opts?.ollamaModel || "";

    setLoadingKi(true);
    try {
      const [aiData, formData] = await Promise.all([
        invoke<BackendSuggestion[]>("check_spelling_ai", {
          path: pdfPath,
          ollamaUrl,
          modelOverride,
        }).catch((e: unknown) => {
          setError(String(e));
          return [] as BackendSuggestion[];
        }),
        invoke<BackendSuggestion[]>("check_formvorschriften", {
          path: pdfPath,
        }).catch(() => [] as BackendSuggestion[]),
      ]);
      setVorschlaege(dedupe([...formData, ...aiData]));
    } finally {
      setLoadingKi(false);
    }
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
    path,
    loadingKi,
    error,
    vorschlaege,
    status,
    analysiere,
    markiere,
    bulk,
    exportiere,
  };
}
