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
      </div>
      <p>
        <span className="alt">{v.original}</span> → <span className="neu">{v.correction}</span>
      </p>
      <small>{v.explanation}</small>
      <div className="aktionen">
        <button className="btn btn-accept btn-sm" onClick={onAccept}>
          Annehmen
        </button>
        <button className="btn btn-reject btn-sm" onClick={onReject}>
          Ablehnen
        </button>
      </div>
    </article>
  );
}
