# PDF Viewer with Highlight Overlays

**Date:** 2026-06-17  
**Status:** Approved

## Summary

Replace the extracted-text `<pre>` panel with a live PDF viewer that renders the original document and overlays colour-coded underlines on each AI suggestion. Accepting a suggestion instantly covers the original word with the corrected text in-view; declining removes the underline.

---

## Goals

- Show the original PDF visually (not extracted text) after a file is loaded
- Yellow underline on every open suggestion
- Green underline + corrected text overlay on accepted suggestions
- Declined suggestions: no overlay
- Works in both single-file and folder mode

## Non-goals

- Text selection inside the PDF viewer
- Zooming / panning (fixed scale for now)
- Printing directly from the viewer

---

## Architecture

### New component: `PdfViewer`

```
<div class="pdf-viewer">           ← position: relative, scrollable
  <div class="pdf-page">          ← one per page, position: relative
    <canvas />                     ← PDF.js renders here
    <div class="pdf-overlay">      ← position: absolute, top:0 left:0, full size
      <div class="highlight …" />  ← one per match, positioned absolutely
    </div>
  </div>
  …
</div>
```

Props:
```ts
interface PdfViewerProps {
  pdfPath: string;           // local OS path
  suggestions: KorrekturVorschlag[];
}
```

### Data flow

1. `pdfPath` changes → load PDF via PDF.js, render all pages to `<canvas>`, extract per-page text layout into `pageLayouts: PageLayout[]`
2. `suggestions` change → recompute highlight rects from `pageLayouts` (no canvas re-render)
3. Accept/reject → React state update → overlay re-renders reactively

### `PageLayout` (internal type)

```ts
interface TextItem {
  str: string;
  startIdx: number;   // character offset in the page's concatenated string
  x: number;          // screen pixels after viewport scale
  y: number;
  w: number;
  h: number;
}

interface PageLayout {
  pageNum: number;
  fullText: string;   // all items concatenated in order
  items: TextItem[];
  viewportWidth: number;
  viewportHeight: number;
}
```

---

## PDF Loading (Tauri)

Add `"core:asset:default"` to `src-tauri/capabilities/default.json`.

In the component, convert the path before loading:

```ts
import { convertFileSrc } from "@tauri-apps/api/core";
const url = convertFileSrc(pdfPath);  // asset://localhost/<path>
const pdf = await pdfjsLib.getDocument(url).promise;
```

PDF.js worker is configured via Vite's `?url` import:

```ts
import workerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
pdfjsLib.GlobalWorkerOptions.workerSrc = workerUrl;
```

---

## Text Matching Algorithm

For each page:

1. Call `page.getTextContent({ includeMarkedContent: false })`
2. Walk items in order, accumulating `startIdx` per item
3. Apply the viewport transform to get screen-pixel coordinates using `viewport.convertToViewportRectangle([x0, y0, x1, y1])` from the pdfjs viewport object — this handles the PDF bottom-up coordinate flip and scale correctly. The raw item `transform[4]`/`transform[5]` are PDF-space coordinates; always go through the viewport API rather than manual matrix math.
4. Store as `PageLayout`

For each suggestion with `status !== "abgelehnt"`:

1. Run `pageLayout.fullText.indexOf(original)` (repeat for all pages until found)
2. Collect all `TextItem`s whose `[startIdx, startIdx + str.length)` overlaps `[matchStart, matchEnd)`
3. Union their bounding boxes → `{ x, y, w, h }` in screen pixels
4. If `original` is `"(fehlt)"` (Formvorschrift missing-item sentinel) → skip highlight (no text to underline)

---

## Highlight Rendering

Each highlight is a `<div>` with `position: absolute`, sized and positioned from the computed bounding box:

```
top:  y - h        (PDF coords: y is baseline, subtract height to get top)
left: x
width: w
height: h
```

**Open (`"offen"`):**
- `border-bottom: 2px solid #facc15` (yellow-400)
- `background: rgba(250, 204, 21, 0.2)`

**Accepted (`"angenommen"`):**
- White `<div>` at same position (`background: var(--color-paper)`) to cover original text
- `<span>` overlaid with correction text, `font-size: h * 0.85px`, `color: #16a34a`
- `border-bottom: 2px solid #16a34a` (green-600)

**Declined (`"abgelehnt"`):** nothing rendered.

---

## Files Changed

| File | Change |
|---|---|
| `package.json` | add `pdfjs-dist` |
| `src-tauri/capabilities/default.json` | add `"core:asset:default"` |
| `src/hooks/useCorrections.ts` | return `path` |
| `src/components/PdfViewer.tsx` | new component |
| `src/App.tsx` | replace `<pre>` with `<PdfViewer>` in both modes |

---

## Edge Cases

- **`"(fehlt)"` sentinel** (Formvorschrift corrections where there is no original text in the PDF): skip the highlight; the correction card still shows in the right panel.
- **Multi-page PDFs**: iterate all pages; first match wins per suggestion.
- **Phrase spanning two text items**: union bounding boxes, draw one highlight rectangle across both.
- **PDF not yet loaded / loading state**: show a spinner in the viewer panel while `pageLayouts` is null.
- **`pdfPath` is empty string** (no file loaded yet): render nothing.
