interface Props {
  disabled: boolean;
  onExport: () => void;
}

export function ExportButton({ disabled, onExport }: Props) {
  return (
    <button disabled={disabled} onClick={onExport}>
      Korrigiertes PDF exportieren
    </button>
  );
}
