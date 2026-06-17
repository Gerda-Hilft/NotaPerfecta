import type { PipelineModus } from "../types/corrections";

interface Props {
  value: PipelineModus;
  onChange: (mode: PipelineModus) => void;
}

export function PipelineToggle({ value, onChange }: Props) {
  return (
    <div className="pipeline-toggle">
      <button className={`chip ${value === "ki" ? "chip-active" : ""}`} onClick={() => onChange("ki")}>
        Nur KI
      </button>
      <button className={`chip ${value === "woerterbuch" ? "chip-active" : ""}`} onClick={() => onChange("woerterbuch")}>
        Nur Wörterbuch
      </button>
      <button className={`chip ${value === "beides" ? "chip-active" : ""}`} onClick={() => onChange("beides")}>
        Beides
      </button>
    </div>
  );
}
