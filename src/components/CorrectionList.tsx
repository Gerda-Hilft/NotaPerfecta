import type { KorrekturVorschlag } from "../types/corrections";
import { SuggestionCard } from "./SuggestionCard";

interface Props {
  suggestions: KorrekturVorschlag[];
  onAccept: (id: string) => void;
  onReject: (id: string) => void;
}

const gruppen = ["Formvorschrift", "Rechtschreibung", "Grammatik", "Zeichensetzung"] as const;

export function CorrectionList({ suggestions, onAccept, onReject }: Props) {
  return (
    <div className="liste">
      {gruppen.map((group) => {
        const items = suggestions.filter((s) => s.type === group);
        if (!items.length) return null;
        return (
          <section key={group}>
            <h3>{group}</h3>
            {items.map((v) => (
              <SuggestionCard key={v.id} v={v} onAccept={() => onAccept(v.id)} onReject={() => onReject(v.id)} />
            ))}
          </section>
        );
      })}
    </div>
  );
}
