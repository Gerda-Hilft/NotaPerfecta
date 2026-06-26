# PDF Viewer with Highlight Overlays Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the extracted-text `<pre>` panel with a live PDF viewer that overlays yellow/green underlines on AI suggestions, and instantly shows corrected text when a suggestion is accepted.

**Architecture:** Use `pdfjs-dist` to render PDF pages to `<canvas>` elements in an imperative loop, extract per-page text layout (item positions in screen pixels), then use React state to drive a separate absolutely-positioned overlay `<div>` containing highlight boxes matched to suggestion strings. Canvas rendering and overlay rendering are decoupled — suggestions can change without re-rendering the PDF.

**Tech Stack:** pdfjs-dist ^4, Vitest (unit tests for pure matching logic), `@tauri-apps/api/core` convertFileSrc (file URL bridging), TypeScript

## Global Constraints

- Tauri v2 (`@tauri-apps/api@^2`), React 18, Vite 6
- No new Rust commands — all PDF rendering is in the frontend
- Scale constant `SCALE = 1.5` — one place to tune zoom
- `"(fehlt)"` sentinel value must never produce a highlight rect
- `"abgelehnt"` suggestions must produce no highlight at all
- yellow underlines: `#facc15` (open), green underlines + text cover: `#16a34a` (accepted)
- `--color-paper` CSS variable for the white-cover on accepted suggestions (falls back to `#fff`)
- German UI strings: "PDF wird geladen…"

---

### Task 1: Install dependencies, configure Vitest, enable Tauri asset capability

**Files:**
- Modify: `package.json`
- Modify: `vite.config.ts`
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Produces: `vitest` available for `pnpm test`; `pdfjs-dist` importable; `convertFileSrc` works in dev/prod

- [ ] **Step 1: Install pdfjs-dist and vitest**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
pnpm add pdfjs-dist
pnpm add -D vitest
```

Expected: `pdfjs-dist` appears in `dependencies`, `vitest` in `devDependencies` in `package.json`.

- [ ] **Step 2: Add vitest config and pdfjs optimizeDeps to vite.config.ts**

Replace the contents of `vite.config.ts` with:

```ts
/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  optimizeDeps: {
    exclude: ["pdfjs-dist"],
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    environment: "node",
  },
}));
```

- [ ] **Step 3: Add vitest script to package.json**

In `package.json`, add to `"scripts"`:
```json
"test": "vitest run"
```

- [ ] **Step 4: Enable Tauri asset protocol capability**

In `src-tauri/capabilities/default.json`, add `"core:asset:default"` to the `permissions` array:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default",
    "core:asset:default"
  ]
}
```

- [ ] **Step 5: Verify build compiles**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
pnpm build
```

Expected: no TypeScript or Vite errors. (Tauri build is not needed at this step.)

- [ ] **Step 6: Commit**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
git add package.json pnpm-lock.yaml vite.config.ts src-tauri/capabilities/default.json
git commit -m "chore: add pdfjs-dist, vitest; enable Tauri asset capability"
```

---

### Task 2: Types and pure text-matching logic (with tests)

**Files:**
- Create: `src/lib/pdfHighlight.ts`
- Create: `src/lib/pdfHighlight.test.ts`

**Interfaces:**
- Consumes: `KorrekturVorschlag` from `../types/corrections`; `PDFPageProxy` from `pdfjs-dist` (import type only)
- Produces:
  - `buildPageLayout(page: PDFPageProxy, scale: number): Promise<PageLayout>` — called by PdfViewer per page
  - `findHighlights(geometries: PageGeometry[], suggestions: KorrekturVorschlag[]): HighlightRect[]` — called by PdfViewer on every render

- [ ] **Step 1: Write the failing tests**

Create `src/lib/pdfHighlight.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { findHighlights } from "./pdfHighlight";
import type { PageGeometry } from "./pdfHighlight";
import type { KorrekturVorschlag } from "../types/corrections";

const geo: PageGeometry = {
  topOffset: 0,
  width: 600,
  height: 800,
  layout: {
    fullText: "Hallo Welt",
    items: [
      { str: "Hallo ", startIdx: 0, x: 10, y: 20, w: 50, h: 12 },
      { str: "Welt",   startIdx: 6, x: 60, y: 20, w: 30, h: 12 },
    ],
    viewportWidth: 600,
    viewportHeight: 800,
  },
};

function s(
  partial: Pick<KorrekturVorschlag, "id" | "original"> &
    Partial<KorrekturVorschlag>
): KorrekturVorschlag {
  return {
    correction: "x",
    type: "Rechtschreibung",
    position: 0,
    explanation: "",
    status: "offen",
    ...partial,
  };
}

describe("findHighlights", () => {
  it("returns a rect for a single-item match", () => {
    const result = findHighlights([geo], [s({ id: "a", original: "Welt" })]);
    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({ suggestionId: "a", x: 60, y: 20, w: 30, h: 12 });
  });

  it("unions rects when match spans two items", () => {
    const result = findHighlights([geo], [s({ id: "b", original: "Hallo Welt" })]);
    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({ x: 10, y: 20, w: 80, h: 12 });
  });

  it("skips the (fehlt) sentinel", () => {
    const result = findHighlights([geo], [s({ id: "c", original: "(fehlt)" })]);
    expect(result).toHaveLength(0);
  });

  it("skips abgelehnt suggestions", () => {
    const result = findHighlights(
      [geo],
      [s({ id: "d", original: "Welt", status: "abgelehnt" })]
    );
    expect(result).toHaveLength(0);
  });

  it("adds topOffset to y coordinate", () => {
    const geoOffset: PageGeometry = { ...geo, topOffset: 100 };
    const result = findHighlights([geoOffset], [s({ id: "e", original: "Welt" })]);
    expect(result[0].y).toBe(120);
  });

  it("returns multiple rects when original appears more than once", () => {
    const repeated: PageGeometry = {
      topOffset: 0,
      width: 600,
      height: 800,
      layout: {
        fullText: "gut gut",
        items: [
          { str: "gut ", startIdx: 0, x: 10, y: 20, w: 20, h: 10 },
          { str: "gut",  startIdx: 4, x: 30, y: 20, w: 20, h: 10 },
        ],
        viewportWidth: 600,
        viewportHeight: 800,
      },
    };
    const result = findHighlights([repeated], [s({ id: "f", original: "gut" })]);
    expect(result).toHaveLength(2);
  });
});
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
pnpm test
```

Expected: FAIL — `findHighlights` not found / module missing.

- [ ] **Step 3: Implement pdfHighlight.ts**

Create `src/lib/pdfHighlight.ts`:

```ts
import type { PDFPageProxy } from "pdfjs-dist";
import type { KorrekturVorschlag } from "../types/corrections";

export interface TextItem {
  str: string;
  startIdx: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface PageLayout {
  fullText: string;
  items: TextItem[];
  viewportWidth: number;
  viewportHeight: number;
}

export interface PageGeometry {
  topOffset: number;
  width: number;
  height: number;
  layout: PageLayout;
}

export interface HighlightRect {
  suggestionId: string;
  x: number;
  y: number;
  w: number;
  h: number;
  fontSize: number;
}

type RawTextItem = {
  str: string;
  transform: number[];
  width: number;
  height: number;
};

export async function buildPageLayout(
  page: PDFPageProxy,
  scale: number
): Promise<PageLayout> {
  const viewport = page.getViewport({ scale });
  const textContent = await page.getTextContent();

  let cursor = 0;
  const items: TextItem[] = [];

  for (const raw of textContent.items) {
    if (!("str" in raw)) continue;
    const item = raw as RawTextItem;
    if (!item.str) continue;

    const { str, transform, width } = item;
    const itemHeight = item.height || Math.abs(transform[0]);

    const [vx1, vy1, vx2, vy2] = viewport.convertToViewportRectangle([
      transform[4],
      transform[5],
      transform[4] + width,
      transform[5] + itemHeight,
    ]);

    items.push({
      str,
      startIdx: cursor,
      x: Math.min(vx1, vx2),
      y: Math.min(vy1, vy2),
      w: Math.abs(vx2 - vx1),
      h: Math.abs(vy2 - vy1),
    });

    cursor += str.length;
  }

  return {
    fullText: items.map((i) => i.str).join(""),
    items,
    viewportWidth: viewport.width,
    viewportHeight: viewport.height,
  };
}

export function findHighlights(
  geometries: PageGeometry[],
  suggestions: KorrekturVorschlag[]
): HighlightRect[] {
  const result: HighlightRect[] = [];

  for (const s of suggestions) {
    if (s.status === "abgelehnt") continue;
    if (s.original === "(fehlt)") continue;

    for (const geo of geometries) {
      const { fullText, items } = geo.layout;
      let from = 0;

      while (from < fullText.length) {
        const matchStart = fullText.indexOf(s.original, from);
        if (matchStart === -1) break;

        const matchEnd = matchStart + s.original.length;
        const hits = items.filter(
          (item) =>
            item.startIdx < matchEnd &&
            item.startIdx + item.str.length > matchStart
        );

        if (hits.length > 0) {
          const x = Math.min(...hits.map((i) => i.x));
          const y = Math.min(...hits.map((i) => i.y)) + geo.topOffset;
          const right = Math.max(...hits.map((i) => i.x + i.w));
          const bottom = Math.max(...hits.map((i) => i.y + i.h)) + geo.topOffset;

          result.push({
            suggestionId: s.id,
            x,
            y,
            w: right - x,
            h: bottom - y,
            fontSize: hits[0].h * 0.85,
          });
        }

        from = matchEnd;
      }
    }
  }

  return result;
}
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
pnpm test
```

Expected: all 6 tests PASS.

- [ ] **Step 5: Commit**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
git add src/lib/pdfHighlight.ts src/lib/pdfHighlight.test.ts
git commit -m "feat: add PDF text layout and highlight matching logic"
```

---

### Task 3: PdfViewer component and CSS

**Files:**
- Create: `src/components/PdfViewer.tsx`
- Modify: `src/App.css`

**Interfaces:**
- Consumes:
  - `buildPageLayout(page, scale): Promise<PageLayout>` from `../lib/pdfHighlight`
  - `findHighlights(geometries, suggestions): HighlightRect[]` from `../lib/pdfHighlight`
  - `KorrekturVorschlag` from `../types/corrections`
  - `convertFileSrc(path: string): string` from `@tauri-apps/api/core`
  - `pdfjsLib` from `pdfjs-dist`
- Produces: `<PdfViewer pdfPath={string} suggestions={KorrekturVorschlag[]} />` — drop-in replacement for the `<pre>` text panel

- [ ] **Step 1: Create PdfViewer.tsx**

Create `src/components/PdfViewer.tsx`:

```tsx
import { useEffect, useRef, useState } from "react";
import * as pdfjsLib from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
import { convertFileSrc } from "@tauri-apps/api/core";
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
  const canvasContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = canvasContainerRef.current;
    if (!pdfPath || !container) {
      setGeometries([]);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setGeometries([]);
    container.innerHTML = "";

    (async () => {
      const pdf = await pdfjsLib.getDocument(convertFileSrc(pdfPath)).promise;
      const newGeometries: PageGeometry[] = [];
      let topOffset = 0;

      for (let i = 1; i <= pdf.numPages; i++) {
        if (cancelled) return;

        const page = await pdf.getPage(i);
        const viewport = page.getViewport({ scale: SCALE });

        const canvas = document.createElement("canvas");
        canvas.width = viewport.width;
        canvas.height = viewport.height;
        canvas.style.display = "block";
        container.appendChild(canvas);

        const ctx = canvas.getContext("2d")!;
        await page.render({ canvasContext: ctx, viewport }).promise;

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
    })();

    return () => {
      cancelled = true;
      container.innerHTML = "";
    };
  }, [pdfPath]);

  const highlights = findHighlights(geometries, suggestions);
  const containerWidth = geometries[0]?.width ?? 0;
  const containerHeight = geometries.reduce(
    (sum, g) => sum + g.height + PAGE_GAP,
    0
  );

  return (
    <div className="pdf-viewer">
      {loading && (
        <div className="pdf-loading">
          <span>PDF wird geladen…</span>
        </div>
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
            }}
          >
            {highlights.map((rect, i) => {
              const s = suggestions.find((s) => s.id === rect.suggestionId)!;
              return s.status === "angenommen" ? (
                <AcceptedHighlight key={i} rect={rect} correction={s.correction} />
              ) : (
                <div
                  key={i}
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
          borderBottom: "2px solid #16a34a",
        }}
      >
        {correction}
      </span>
    </div>
  );
}
```

- [ ] **Step 2: Add CSS for PdfViewer to App.css**

Append to `src/App.css`:

```css
/* ── PDF Viewer ─────────────────────────────────────────── */
.pdf-viewer {
  overflow: auto;
  flex: 1;
  padding: 1rem;
  background: var(--color-bg);
}

.pdf-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: var(--color-muted, #888);
}

.pdf-overlay {
  pointer-events: none;
}

.highlight-open {
  border-bottom: 2px solid #facc15;
  background: rgba(250, 204, 21, 0.2);
}
```

- [ ] **Step 3: Verify TypeScript compiles**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
pnpm build
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
git add src/components/PdfViewer.tsx src/App.css
git commit -m "feat: add PdfViewer component with highlight overlay"
```

---

### Task 4: Expose `path` from useCorrections and wire PdfViewer into App

**Files:**
- Modify: `src/hooks/useCorrections.ts:91-101`
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: `PdfViewer` from `./components/PdfViewer`
- Produces: final working feature — PDF visible in both single-file and folder modes, with live highlights

- [ ] **Step 1: Return `path` from useCorrections**

In `src/hooks/useCorrections.ts`, the `path` state variable is already tracked (line 31) but not returned. Add it to the return object at the bottom of the function:

```ts
  return {
    text,
    path,       // ← add this line
    loadingKi,
    error,
    vorschlaege,
    status,
    analysiere,
    markiere,
    bulk,
    exportiere,
  };
```

- [ ] **Step 2: Replace `<pre>` with `<PdfViewer>` in single-file mode in App.tsx**

In `src/App.tsx`:

1. Add the import at the top:
```ts
import { PdfViewer } from "./components/PdfViewer";
```

2. In the single-file section (around line 169), replace:
```tsx
              <article className="panel">
                <h2>Originaltext</h2>
                <pre>{single.text || "Noch kein Zeugnis geladen."}</pre>
              </article>
```
with:
```tsx
              <article className="panel">
                <h2>Originaltext</h2>
                {single.path ? (
                  <PdfViewer pdfPath={single.path} suggestions={single.vorschlaege} />
                ) : (
                  <p style={{ color: "var(--color-muted, #888)", padding: "1rem" }}>
                    Noch kein Zeugnis geladen.
                  </p>
                )}
              </article>
```

- [ ] **Step 3: Replace `<pre>` with `<PdfViewer>` in folder mode in App.tsx**

In the folder mode section (around line 117), replace:
```tsx
                  <article className="panel folder-panel">
                    <h2>Originaltext — {current.filename}</h2>
                    <pre>{current.text || "Kein Text extrahiert."}</pre>
                  </article>
```
with:
```tsx
                  <article className="panel folder-panel">
                    <h2>Originaltext — {current.filename}</h2>
                    <PdfViewer pdfPath={current.path} suggestions={current.vorschlaege} />
                  </article>
```

- [ ] **Step 4: Build and verify no TypeScript errors**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
pnpm build
```

Expected: clean build, no errors.

- [ ] **Step 5: Run the app and manually verify**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
pnpm tauri dev
```

Manual test checklist:
- [ ] Drop a PDF → PDF renders visually in the left panel (not plain text)
- [ ] Open suggestions have yellow underlines on the matching words
- [ ] Clicking "Annehmen" on a suggestion: the original word disappears, green correction text appears in its place, underline turns green
- [ ] Clicking "Ablehnen" on a suggestion: the highlight disappears entirely
- [ ] "Formvorschrift" suggestions with `original === "(fehlt)"` show no underline in the PDF panel
- [ ] Loading a second PDF replaces the first correctly (no leftover canvases)
- [ ] Folder mode: selecting a file from the sidebar shows its PDF in the left panel with live highlights

- [ ] **Step 6: Commit**

```bash
cd "/home/me/Projects/Multi lang/NotaPerfecta"
git add src/hooks/useCorrections.ts src/App.tsx
git commit -m "feat: replace text panel with PDF viewer and live highlight overlays"
```
