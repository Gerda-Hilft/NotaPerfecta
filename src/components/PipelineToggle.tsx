import type { PipelineModus } from "../types/corrections";

interface Props {
  value: PipelineModus;
  onChange: (mode: PipelineModus) => void;
}

export function PipelineToggle({ value, onChange }: Props) {
  return (
    <div className="pipeline-toggle">
      <button className={value === "ki" ? "aktiv" : ""} onClick={() => onChange("ki")}>
        Nur KI
      </button>
      <button className={value === "woerterbuch" ? "aktiv" : ""} onClick={() => onChange("woerterbuch")}>
        Nur Wörterbuch
      </button>
      <button className={value === "beides" ? "aktiv" : ""} onClick={() => onChange("beides")}>
        Beides
      </button>
    </div>
  );
}
