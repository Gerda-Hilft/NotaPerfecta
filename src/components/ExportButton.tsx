interface Props {
  disabled: boolean;
  onExport: () => void;
}

export function ExportButton({ disabled, onExport }: Props) {
  return (
    <button className="btn btn-primary btn-sm" disabled={disabled} onClick={onExport}>
      Korrigiertes PDF exportieren
    </button>
  );
}
