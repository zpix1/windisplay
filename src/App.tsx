import "./App.css";
import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Resolution = {
  width: number;
  height: number;
  bits_per_pixel: number;
  refresh_hz: number;
};

type DisplayInfo = {
  device_name: string;
  friendly_name: string;
  is_primary: boolean;
  position_x: number;
  position_y: number;
  current: Resolution;
  modes: Resolution[];
};

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

  const selected = useMemo(() => monitors[selectedIndex], [monitors, selectedIndex]);
  const [selectedResKey, setSelectedResKey] = useState<string>("");

  useEffect(() => {
    if (selected) {
      setSelectedResKey(`${selected.current.width}x${selected.current.height}`);
    } else {
      setSelectedResKey("");
    }
  }, [selected]);

  const resolutionOptions = useMemo(() => {
    if (!selected) return [] as { key: string; width: number; height: number }[];
    const set = new Map<string, { key: string; width: number; height: number }>();
    for (const m of selected.modes) {
      const key = `${m.width}x${m.height}`;
      if (!set.has(key)) {
        set.set(key, { key, width: m.width, height: m.height });
      }
    }
    return Array.from(set.values()).sort((a, b) => b.width * b.height - a.width * a.height);
  }, [selected]);

  return (
    <div className="app-root">
      <div className="section">
        <label className="label" htmlFor="monitor-select">
          Monitor
        </label>
        <select
          id="monitor-select"
          className="select"
          disabled={loading || monitors.length === 0}
          value={selectedIndex}
          onChange={(e) => setSelectedIndex(Number(e.target.value))}
        >
          {monitors.map((m, idx) => (
            <option value={idx} key={m.device_name}>
              {m.friendly_name || m.device_name}
              {m.is_primary ? " (Primary)" : ""}
            </option>
          ))}
        </select>
      </div>

      {error && <div className="error">{error}</div>}

      <div className="section">
        <label className="label" htmlFor="resolution-select">
          Resolution
        </label>
        <select
          id="resolution-select"
          className="select"
          disabled={loading || !selected || resolutionOptions.length === 0}
          value={selectedResKey}
          onChange={(e) => setSelectedResKey(e.target.value)}
        >
          {resolutionOptions.map((opt) => (
            <option value={opt.key} key={opt.key}>
              {opt.width} × {opt.height}
            </option>
          ))}
        </select>
      </div>

      <div className="section">
        {loading && <div className="muted">Loading...</div>}
        {!loading && !selected && <div className="muted">No monitors detected</div>}
        {!loading && selected && (
          <div className="details">
            <div className="row">
              <span className="key">Resolution:</span>
              <span className="value">
                {selected.current.width} × {selected.current.height}
              </span>
            </div>
            <div className="row">
              <span className="key">Refresh rate:</span>
              <span className="value">{selected.current.refresh_hz} Hz</span>
            </div>
            {selectedResKey && (
              <div className="row">
                <span className="key">Selected:</span>
                <span className="value">{selectedResKey.replace("x", " × ")}</span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
