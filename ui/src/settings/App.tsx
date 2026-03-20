import { useState, useEffect, useCallback, useRef } from "react";
import type { SettingsData, ThresholdConfig, ThresholdAction } from "@shared/types";
import { Card } from "@shared/Card";
import { Button } from "@shared/Button";
import "./App.css";

const defaultThreshold = (
  warning: number,
  critical: number,
): ThresholdConfig => ({
  warning,
  warning_action: "notify",
  critical,
  critical_action: "notify",
});

const defaultSettings: SettingsData = {
  memory_load: defaultThreshold(80, 95),
  available_memory: defaultThreshold(4096, 2048),
  modified_list: defaultThreshold(1024, 4096),
  standby_list: defaultThreshold(2048, 8192),
};

interface AreaDef {
  key: keyof SettingsData;
  label: string;
  hint: string;
  min: number;
  max?: number;
  step: number;
}

const areas: AreaDef[] = [
  { key: "memory_load", label: "Memory Load", hint: "Overall usage (%)", min: 1, max: 100, step: 1 },
  { key: "available_memory", label: "Available Memory", hint: "Alert below (MB)", min: 0, step: 256 },
  { key: "modified_list", label: "Modified List", hint: "Dirty pages (MB)", min: 0, step: 128 },
  { key: "standby_list", label: "Standby List", hint: "Cached pages (MB)", min: 0, step: 256 },
];

const actions: { value: ThresholdAction; label: string }[] = [
  { value: "none", label: "None" },
  { value: "notify", label: "Notify" },
  { value: "purge", label: "Purge" },
];

export function App() {
  const [settings, setSettings] = useState<SettingsData>(defaultSettings);
  const [toast, setToast] = useState<string | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout>>();

  const handleLoad = useCallback((data: SettingsData) => {
    setSettings(data);
  }, []);

  const handleToast = useCallback((msg: string) => {
    setToast(msg);
    clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 2000);
  }, []);

  useEffect(() => {
    window.loadSettings = handleLoad;
    window.showToast = handleToast;
    return () => {
      window.loadSettings = undefined;
      window.showToast = undefined;
    };
  }, [handleLoad, handleToast]);

  const update = (
    key: keyof SettingsData,
    field: keyof ThresholdConfig,
    value: string,
  ) => {
    setSettings((prev) => ({
      ...prev,
      [key]: {
        ...prev[key],
        [field]:
          field === "warning_action" || field === "critical_action"
            ? value
            : parseFloat(value) || 0,
      },
    }));
  };

  const doSave = () => {
    window.ipc.postMessage(
      JSON.stringify({ cmd: "save", settings }),
    );
  };

  const doCancel = () => {
    window.ipc.postMessage(JSON.stringify({ cmd: "cancel" }));
  };

  return (
    <>
      <h1>Settings</h1>
      <p className="subtitle">
        Configure warning thresholds and automated actions per memory area.
      </p>

      <Card title="Thresholds">
        <div className="header-row">
          <div className="col-header">Area</div>
          <div className="col-header">Warning</div>
          <div className="col-header">Action</div>
          <div className="col-header">Critical</div>
          <div className="col-header">Action</div>
        </div>

        {areas.map((area) => (
          <div className="setting-row" key={area.key}>
            <div>
              <div className="setting-label">{area.label}</div>
              <div className="setting-hint">{area.hint}</div>
            </div>
            <input
              type="number"
              min={area.min}
              max={area.max}
              step={area.step}
              value={settings[area.key].warning}
              onChange={(e) => update(area.key, "warning", e.target.value)}
            />
            <select
              value={settings[area.key].warning_action}
              onChange={(e) =>
                update(area.key, "warning_action", e.target.value)
              }
            >
              {actions.map((a) => (
                <option key={a.value} value={a.value}>
                  {a.label}
                </option>
              ))}
            </select>
            <input
              type="number"
              min={area.min}
              max={area.max}
              step={area.step}
              value={settings[area.key].critical}
              onChange={(e) => update(area.key, "critical", e.target.value)}
            />
            <select
              value={settings[area.key].critical_action}
              onChange={(e) =>
                update(area.key, "critical_action", e.target.value)
              }
            >
              {actions.map((a) => (
                <option key={a.value} value={a.value}>
                  {a.label}
                </option>
              ))}
            </select>
          </div>
        ))}
      </Card>

      <div className="btn-row">
        <Button onClick={doCancel}>Cancel</Button>
        <Button primary onClick={doSave}>
          Save
        </Button>
      </div>

      <div className={`toast-msg${toast ? " show" : ""}`}>
        {toast}
      </div>
    </>
  );
}
