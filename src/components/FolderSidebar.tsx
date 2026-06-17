import { type DateiEintrag, istFertig } from "../hooks/useFolderSession";

interface Props {
  ordnerName: string;
  dateien: DateiEintrag[];
  aktuellerPfad: string | null;
  exportStatus: string | null;
  onSelect: (path: string) => void;
  onExportAll: () => void;
  onClose: () => void;
}

function StatusBadge({ datei }: { datei: DateiEintrag }) {
  switch (datei.phase) {
    case "wartend":
      return <span className="sidebar-badge sidebar-badge-muted">wartend</span>;
    case "wird-analysiert":
      return <span className="sidebar-badge sidebar-badge-loading">…</span>;
    case "fehler":
      return <span className="sidebar-badge sidebar-badge-error">Fehler</span>;
    case "analysiert": {
      const offen = datei.vorschlaege.filter((v) => v.status === "offen").length;
      const total = datei.vorschlaege.length;
      if (total === 0) return <span className="sidebar-badge sidebar-badge-clean">OK</span>;
      if (offen === 0) {
        const angenommen = datei.vorschlaege.filter((v) => v.status === "angenommen").length;
        return <span className="sidebar-badge sidebar-badge-done">{angenommen} fix</span>;
      }
      return <span className="sidebar-badge sidebar-badge-open">{offen} offen</span>;
    }
  }
}

export function FolderSidebar({
  ordnerName,
  dateien,
  aktuellerPfad,
  exportStatus,
  onSelect,
  onExportAll,
  onClose,
}: Props) {
  const offen = dateien.filter((d) => !istFertig(d));
  const fertig = dateien.filter((d) => istFertig(d));

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <span className="sidebar-title" title={ordnerName}>
          {ordnerName.split(/[/\\]/).pop()}
        </span>
        <button className="dialog-close" onClick={onClose} aria-label="Ordner schließen">
          &times;
        </button>
      </div>

      <div className="sidebar-list">
        {offen.length > 0 && (
          <div className="sidebar-section">
            <h4 className="sidebar-section-title">
              Offen <span className="sidebar-count">{offen.length}</span>
            </h4>
            {offen.map((d) => (
              <button
                key={d.path}
                className={`sidebar-item ${d.path === aktuellerPfad ? "sidebar-item-active" : ""}`}
                onClick={() => onSelect(d.path)}
              >
                <span className="sidebar-item-name">{d.filename}</span>
                <StatusBadge datei={d} />
              </button>
            ))}
          </div>
        )}

        {fertig.length > 0 && (
          <div className="sidebar-section">
            <h4 className="sidebar-section-title">
              Fertig <span className="sidebar-count">{fertig.length}</span>
            </h4>
            {fertig.map((d) => (
              <button
                key={d.path}
                className={`sidebar-item sidebar-item-done ${d.path === aktuellerPfad ? "sidebar-item-active" : ""}`}
                onClick={() => onSelect(d.path)}
              >
                <span className="sidebar-item-name">{d.filename}</span>
                <StatusBadge datei={d} />
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="sidebar-footer">
        {exportStatus && <p className="sidebar-export-status">{exportStatus}</p>}
        <button
          className="btn btn-primary btn-sm sidebar-export-btn"
          onClick={onExportAll}
        >
          Alle exportieren → korrekturen/
        </button>
      </div>
    </aside>
  );
}
