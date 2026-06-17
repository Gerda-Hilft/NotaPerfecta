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
