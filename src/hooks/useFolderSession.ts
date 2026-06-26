import { invoke } from "@tauri-apps/api/core";
import { useState, useCallback } from "react";
import type { ExportKorrektur, ExportResult, KorrekturVorschlag } from "../types/corrections";

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

export type DateiPhase = "wartend" | "wird-analysiert" | "analysiert" | "fehler";

export interface DateiEintrag {
  path: string;
  filename: string;
  phase: DateiPhase;
  vorschlaege: KorrekturVorschlag[];
  error: string | null;
}

export function istFertig(d: DateiEintrag): boolean {
  if (d.phase !== "analysiert") return false;
  return d.vorschlaege.length === 0 || d.vorschlaege.every((v) => v.status !== "offen");
}

interface AnalyseOpts {
  ollamaUrl: string;
  ollamaModel: string;
}

export function useFolderSession() {
  const [ordner, setOrdner] = useState<string | null>(null);
  const [dateien, setDateien] = useState<DateiEintrag[]>([]);
  const [aktuellerPfad, setAktuellerPfad] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [exportStatus, setExportStatus] = useState<string | null>(null);

  const aktuell = dateien.find((d) => d.path === aktuellerPfad) ?? null;

  async function oeffneOrdner(pfad: string) {
    const files = await invoke<string[]>("list_pdf_files", { directory: pfad });
    if (files.length === 0) throw new Error("Keine PDF-Dateien im Ordner gefunden.");

    setOrdner(pfad);
    setExportStatus(null);
    setDateien(
      files.map((f) => ({
        path: f,
        filename: f.split(/[/\\]/).pop() || f,
        phase: "wartend" as const,
        vorschlaege: [],
        error: null,
      })),
    );
    setAktuellerPfad(null);
  }

  async function waehleDatei(pfad: string, opts: AnalyseOpts) {
    setAktuellerPfad(pfad);

    const datei = dateien.find((d) => d.path === pfad);
    if (!datei || datei.phase !== "wartend") return;

    setLoading(true);
    setDateien((prev) =>
      prev.map((d) => (d.path === pfad ? { ...d, phase: "wird-analysiert" as const } : d)),
    );

    try {
      const [aiSuggestions, formSuggestions] = await Promise.all([
        invoke<BackendSuggestion[]>("check_spelling_ai", {
          path: pfad,
          ollamaUrl: opts.ollamaUrl,
          modelOverride: opts.ollamaModel,
        }).catch(() => [] as BackendSuggestion[]),
        invoke<BackendSuggestion[]>("check_formvorschriften", {
          path: pfad,
        }).catch(() => [] as BackendSuggestion[]),
      ]);
      const suggestions = [...formSuggestions, ...aiSuggestions];

      setDateien((prev) =>
        prev.map((d) =>
          d.path === pfad
            ? { ...d, phase: "analysiert" as const, vorschlaege: dedupe(suggestions), error: null }
            : d,
        ),
      );
    } catch (e) {
      setDateien((prev) =>
        prev.map((d) =>
          d.path === pfad ? { ...d, phase: "fehler" as const, error: String(e) } : d,
        ),
      );
    } finally {
      setLoading(false);
    }
  }

  const markiere = useCallback(
    (id: string, statusWert: "angenommen" | "abgelehnt") => {
      if (!aktuellerPfad) return;
      setDateien((prev) =>
        prev.map((d) =>
          d.path === aktuellerPfad
            ? {
                ...d,
                vorschlaege: d.vorschlaege.map((v) =>
                  v.id === id ? { ...v, status: statusWert } : v,
                ),
              }
            : d,
        ),
      );
    },
    [aktuellerPfad],
  );

  const bulk = useCallback(
    (statusWert: "angenommen" | "abgelehnt") => {
      if (!aktuellerPfad) return;
      setDateien((prev) =>
        prev.map((d) =>
          d.path === aktuellerPfad
            ? { ...d, vorschlaege: d.vorschlaege.map((v) => ({ ...v, status: statusWert })) }
            : d,
        ),
      );
    },
    [aktuellerPfad],
  );

  async function exportiereAlle() {
    if (!ordner) return;
    const outputDir = `${ordner}/korrekturen`;

    let korrigiert = 0;
    let kopiert = 0;
    let errors = 0;
    let nichtGefunden = 0;

    for (const datei of dateien) {
      const accepted: ExportKorrektur[] =
        datei.phase === "analysiert"
          ? datei.vorschlaege
              .filter((v) => v.status === "angenommen")
              .map(({ original, correction, position }) => ({ original, correction, position }))
          : [];

      try {
        if (accepted.length > 0) {
          const result = await invoke<ExportResult>("export_corrected_pdf", {
            originalPath: datei.path,
            acceptedCorrections: accepted,
            outputDir,
          });
          if (result.applied > 0) {
            korrigiert++;
          } else {
            kopiert++;
          }
          nichtGefunden += result.requested - result.applied;
        } else {
          await invoke<string>("copy_file", {
            source: datei.path,
            destDir: outputDir,
          });
          kopiert++;
        }
      } catch {
        errors++;
      }
    }

    const total = korrigiert + kopiert;
    const parts: string[] = [];
    if (korrigiert > 0) parts.push(`${korrigiert} korrigiert`);
    if (kopiert > 0) parts.push(`${kopiert} unverändert`);
    if (errors > 0) parts.push(`${errors} fehlgeschlagen`);
    if (nichtGefunden > 0) parts.push(`${nichtGefunden} Korrekturen nicht gefunden`);
    setExportStatus(`${total} Dateien → korrekturen/ (${parts.join(", ")})`);
  }

  function schliesseOrdner() {
    setOrdner(null);
    setDateien([]);
    setAktuellerPfad(null);
    setExportStatus(null);
  }

  return {
    ordner,
    dateien,
    aktuell,
    aktuellerPfad,
    loading,
    exportStatus,
    oeffneOrdner,
    waehleDatei,
    markiere,
    bulk,
    exportiereAlle,
    schliesseOrdner,
  };
}
