export type PipelineModus = "ki" | "woerterbuch" | "beides";

export interface KorrekturVorschlag {
  id: string;
  original: string;
  correction: string;
  type: "Rechtschreibung" | "Grammatik" | "Zeichensetzung";
  position: number;
  explanation: string;
  source: "KI" | "Wörterbuch";
  status: "offen" | "angenommen" | "abgelehnt";
}

export interface ExportKorrektur {
  original: string;
  correction: string;
  position: number;
}
