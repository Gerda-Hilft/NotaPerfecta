import { useEffect, useRef, useState } from "react";
import * as pdfjsLib from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
import { invoke } from "@tauri-apps/api/core";
import type { KorrekturVorschlag } from "../types/corrections";
import { buildPageLayout, findHighlights } from "../lib/pdfHighlight";
import type { HighlightRect, PageGeometry } from "../lib/pdfHighlight";

pdfjsLib.GlobalWorkerOptions.workerSrc = workerUrl;

const SCALE = 1.5;
const PAGE_GAP = 8;

interface Props {
  pdfPath: string;
  suggestions: KorrekturVorschlag[];
}

export function PdfViewer({ pdfPath, suggestions }: Props) {
  const [geometries, setGeometries] = useState<PageGeometry[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const canvasContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = canvasContainerRef.current;
    if (!pdfPath || !container) {
      setGeometries([]);
      return;
    }

    let cancelled = false;
    setLoadError(null);
    setLoading(true);
    setGeometries([]);
    container.innerHTML = "";

    const runId = Math.random().toString(36).slice(2, 7);
    console.debug(`[PdfViewer ${runId}] start`, pdfPath);

    (async () => {
      try {
        const bytes = await invoke<number[]>("read_pdf_bytes", { path: pdfPath });
        console.debug(`[PdfViewer ${runId}] got ${bytes.length} bytes, cancelled=${cancelled}`);
        const pdf = await pdfjsLib.getDocument({ data: new Uint8Array(bytes) }).promise;
        console.debug(`[PdfViewer ${runId}] doc loaded, ${pdf.numPages} pages, cancelled=${cancelled}`);
        const newGeometries: PageGeometry[] = [];
        let topOffset = 0;

        for (let i = 1; i <= pdf.numPages; i++) {
          if (cancelled) {
            console.debug(`[PdfViewer ${runId}] cancelled before page ${i}`);
            return;
          }

          const page = await pdf.getPage(i);
          const viewport = page.getViewport({ scale: SCALE });

          const canvas = document.createElement("canvas");
          canvas.width = viewport.width;
          canvas.height = viewport.height;
          canvas.style.display = "block";
          container.appendChild(canvas);

          const ctx = canvas.getContext("2d")!;
          console.debug(`[PdfViewer ${runId}] rendering page ${i}, cancelled=${cancelled}`);
          await page.render({ canvas, canvasContext: ctx, viewport }).promise;
          console.debug(`[PdfViewer ${runId}] rendered page ${i}, cancelled=${cancelled}`);

          const layout = await buildPageLayout(page, SCALE);
          newGeometries.push({
            topOffset,
            width: viewport.width,
            height: viewport.height,
            layout,
          });
          topOffset += viewport.height + PAGE_GAP;
        }

        if (!cancelled) {
          setGeometries(newGeometries);
          setLoading(false);
        }
      } catch (e) {
        console.error(`[PdfViewer ${runId}] FAILED (cancelled=${cancelled}):`, e);
        if (!cancelled) {
          setLoading(false);
          setLoadError(`PDF konnte nicht geladen werden: ${e instanceof Error ? e.message : String(e)}`);
        }
      }
    })();

    return () => {
      cancelled = true;
      container.innerHTML = "";
    };
  }, [pdfPath]);

  const highlights = findHighlights(geometries, suggestions);
  const containerWidth = geometries[0]?.width ?? 0;
  const containerHeight = geometries.reduce(
    (sum, g, i) => sum + g.height + (i < geometries.length - 1 ? PAGE_GAP : 0),
    0
  );

  return (
    <div className="pdf-viewer">
      {loading && (
        <div className="pdf-loading">
          <span>PDF wird geladen…</span>
        </div>
      )}
      {loadError && (
        <p style={{ color: "var(--color-error, #ef4444)", padding: "1rem" }}>
          {loadError}
        </p>
      )}
      <div style={{ position: "relative", width: containerWidth || "auto" }}>
        <div ref={canvasContainerRef} />
        {geometries.length > 0 && (
          <div
            className="pdf-overlay"
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: containerWidth,
              height: containerHeight,
              pointerEvents: "none",
              zIndex: 2,
            }}
          >
            {highlights.map((rect, i) => {
              const s = suggestions.find((s) => s.id === rect.suggestionId)!;
              return s.status === "angenommen" ? (
                <AcceptedHighlight key={`${rect.suggestionId}-${i}`} rect={rect} correction={s.correction} />
              ) : (
                <div
                  key={`${rect.suggestionId}-${i}`}
                  className="highlight-open"
                  style={{
                    position: "absolute",
                    left: rect.x,
                    top: rect.y,
                    width: rect.w,
                    height: rect.h,
                  }}
                />
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function AcceptedHighlight({
  rect,
  correction,
}: {
  rect: HighlightRect;
  correction: string;
}) {
  return (
    <div
      style={{
        position: "absolute",
        left: rect.x,
        top: rect.y,
        width: rect.w,
        height: rect.h,
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: "var(--color-paper, #fff)",
          outline: "2px solid rgba(22, 163, 74, 0.7)",
          borderRadius: 2,
        }}
      />
      <span
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          fontSize: rect.fontSize,
          lineHeight: 1,
          color: "#16a34a",
          whiteSpace: "nowrap",
          fontWeight: "bold",
        }}
      >
        {correction}
      </span>
    </div>
  );
}
