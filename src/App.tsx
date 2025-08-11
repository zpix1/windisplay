import "./App.css";
import { useCallback, useMemo, useState } from "react";
import MonitorControls from "./components/MonitorControls";
import { useMonitorsContext } from "./context/MonitorsContext";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const { monitors, loading, error, setError } = useMonitorsContext();
  const [selectedDeviceName, setSelectedDeviceName] = useState<string | null>(
    null
  );
  const selected = useMemo(
    () =>
      selectedDeviceName
        ? monitors.find((m) => m.device_name === selectedDeviceName)
        : monitors[0],
    [monitors, selectedDeviceName]
  );

  const handleIdentifyMonitors = useCallback(async () => {
    try {
      await invoke("identify_monitors");
    } catch (e) {
      setError((e as Error).message ?? String(e));
    }
  }, [setError]);

  return (
    <div className="app-root">
      <div className="section">
        <label className="label" htmlFor="monitor-select">
          Monitor
        </label>
        <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
          <select
            id="monitor-select"
            className="select"
            disabled={loading || monitors.length === 0}
            value={selectedDeviceName ?? monitors[0]?.device_name ?? ""}
            onChange={(e) => setSelectedDeviceName(e.target.value)}
            style={{ flex: 1 }}
          >
            {monitors.map((m, idx) => (
              <option value={m.device_name} key={m.device_name}>
                {`Monitor ${idx + 1}${m.is_primary ? " (Primary)" : ""}`}
              </option>
            ))}
          </select>
          <button
            className="identify-button"
            onClick={handleIdentifyMonitors}
            disabled={loading || monitors.length === 0}
            title="Identify monitors by showing numbers on each screen"
          >
            🔍
          </button>
        </div>
      </div>

      {error && <div className="error">{error}</div>}

      <div className="section">
        {loading && <div className="muted">Loading...</div>}
        {!loading && !selected && (
          <div className="muted">No monitors detected</div>
        )}
      </div>

      {!loading && selected && (
        <MonitorControls
          monitor={selected}
          disabled={loading}
          onError={(msg) => setError(msg)}
        />
      )}
    </div>
  );
}

export default App;
