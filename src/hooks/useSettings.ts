import { useState, useEffect, useCallback } from "react";

export interface AppSettings {
  ollamaUrl: string;
  ollamaModel: string;
}

const STORAGE_KEY = "notaperfecta-settings";

const DEFAULTS: AppSettings = {
  ollamaUrl: "http://127.0.0.1:11434",
  ollamaModel: "",
};

function load(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULTS;
    return { ...DEFAULTS, ...JSON.parse(raw) };
  } catch {
    return DEFAULTS;
  }
}

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(load);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }, [settings]);

  const update = useCallback((patch: Partial<AppSettings>) => {
    setSettings((prev) => ({ ...prev, ...patch }));
  }, []);

  const reset = useCallback(() => setSettings(DEFAULTS), []);

  return { settings, update, reset };
}
