import "./App.css";
import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import MonitorControls from "./components/MonitorControls";
import { DisplayInfo } from "./lib/Resolutions";

function App() {
  const [monitors, setMonitors] = useState<DisplayInfo[]>([]);
  const [selectedIndex, setSelectedIndex] = useState<number>(0);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setLoading(true);
        const result = await invoke<DisplayInfo[]>("get_all_monitors");
        if (!cancelled) {
          setMonitors(result ?? []);
          setSelectedIndex(0);
        }
      } catch (e) {
        if (!cancelled) setError((e as Error).message ?? String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const selected = useMemo(
    () => monitors[selectedIndex],
    [monitors, selectedIndex]
  );

  const handleResolutionChanged = useCallback(async () => {
    // Refresh monitors after resolution change
    try {
      const result = await invoke<DisplayInfo[]>("get_all_monitors");
      setMonitors(result ?? []);
      // Keep selection on same device
      const idx = (result ?? []).findIndex(
        (m) => m.device_name === selected?.device_name
      );
      setSelectedIndex(idx >= 0 ? idx : 0);
    } catch (e) {
      setError((e as Error).message ?? String(e));
    }
  }, [selected?.device_name]);

  const handleIdentifyMonitors = useCallback(async () => {
    try {
      await invoke("identify_monitors");
    } catch (e) {
      setError((e as Error).message ?? String(e));
    }
  }, []);

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
            value={selectedIndex}
            onChange={(e) => setSelectedIndex(Number(e.target.value))}
            style={{ flex: 1 }}
          >
            {monitors.map((m, idx) => (
              <option value={idx} key={m.device_name}>
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
          onResolutionChanged={handleResolutionChanged}
        />
      )}
    </div>
  );
}

export default App;
