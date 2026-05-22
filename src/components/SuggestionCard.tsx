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
        <strong>{v.type}</strong>
        <span className="badge">{v.source}</span>
      </div>
      <p>
        <span className="alt">{v.original}</span> → <span className="neu">{v.correction}</span>
      </p>
      <small>{v.explanation}</small>
      <div className="aktionen">
        <button onClick={onAccept}>✓ Annehmen</button>
        <button onClick={onReject}>✗ Ablehnen</button>
      </div>
    </article>
  );
}
