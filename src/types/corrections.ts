export interface KorrekturVorschlag {
  id: string;
  original: string;
  correction: string;
  type: "Rechtschreibung" | "Grammatik" | "Zeichensetzung" | "Formvorschrift";
  position: number;
  explanation: string;
  status: "offen" | "angenommen" | "abgelehnt";
}

export interface ExportKorrektur {
  original: string;
  correction: string;
  position: number;
}
