import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../hooks/useSettings";

interface Props {
  open: boolean;
  settings: AppSettings;
  onUpdate: (patch: Partial<AppSettings>) => void;
  onReset: () => void;
  onClose: () => void;
}

export function SettingsDialog({ open, settings, onUpdate, onReset, onClose }: Props) {
  const [models, setModels] = useState<string[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [modelsError, setModelsError] = useState("");

  useEffect(() => {
    if (open) fetchModels();
  }, [open, settings.ollamaUrl]);

  async function fetchModels() {
    setModelsLoading(true);
    setModelsError("");
    try {
      const list = await invoke<string[]>("list_ollama_models", { ollamaUrl: settings.ollamaUrl });
      setModels(list);
    } catch (e) {
      setModelsError(String(e));
      setModels([]);
    } finally {
      setModelsLoading(false);
    }
  }

  if (!open) return null;

  return (
    <div className="dialog-backdrop" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Einstellungen</h2>
          <button className="dialog-close" onClick={onClose} aria-label="Schließen">
            &times;
          </button>
        </div>

        <div className="dialog-body">
          <fieldset className="settings-group">
            <legend>Ollama</legend>

            <label className="settings-label">
              Server-URL
              <input
                className="settings-input"
                type="text"
                value={settings.ollamaUrl}
                onChange={(e) => onUpdate({ ollamaUrl: e.target.value })}
                placeholder="http://127.0.0.1:11434"
              />
            </label>

            <label className="settings-label">
              Modell
              <div className="settings-row">
                <select
                  className="settings-select"
                  value={settings.ollamaModel}
                  onChange={(e) => onUpdate({ ollamaModel: e.target.value })}
                >
                  <option value="">Automatisch (bevorzugt llama3)</option>
                  {models.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
                <button
                  className="btn btn-outline btn-sm"
                  onClick={fetchModels}
                  disabled={modelsLoading}
                >
                  {modelsLoading ? "…" : "Aktualisieren"}
                </button>
              </div>
              {modelsError && <span className="settings-hint settings-hint-error">{modelsError}</span>}
            </label>
          </fieldset>

        </div>

        <div className="dialog-footer">
          <button className="btn btn-outline btn-sm" onClick={onReset}>
            Zurücksetzen
          </button>
          <button className="btn btn-primary btn-sm" onClick={onClose}>
            Fertig
          </button>
        </div>
      </div>
    </div>
  );
}
