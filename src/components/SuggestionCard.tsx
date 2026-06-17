import type { KorrekturVorschlag } from "../types/corrections";

interface Props {
  v: KorrekturVorschlag;
  onAccept: () => void;
  onReject: () => void;
}

export function SuggestionCard({ v, onAccept, onReject }: Props) {
  return (
    <article className={`karte ${v.status}`}>
      <div className="karte-kopf">
        <span className="badge">{v.type}</span>
        <span className="badge badge-muted">{v.source}</span>
      </div>
      <p>
        <span className="alt">{v.original}</span> → <span className="neu">{v.correction}</span>
      </p>
      <small>{v.explanation}</small>
      <div className="aktionen">
        <button className="btn btn-outline btn-sm" onClick={onAccept}>
          ✓ Annehmen
        </button>
        <button className="btn btn-ghost btn-sm" onClick={onReject}>
          ✗ Ablehnen
        </button>
      </div>
    </article>
  );
}
