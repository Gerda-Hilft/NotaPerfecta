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
    if (!s.original) continue;

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
