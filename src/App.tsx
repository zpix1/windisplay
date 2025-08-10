import "./App.css";
import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ResolutionSelect from "./components/ResolutionSelect";
import BrightnessSlider from "./components/BrightnessSlider";

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

  // resolution options are now computed inside ResolutionSelect

  async function applyResolution(key: string) {
    if (!selected) return;
    const [wStr, hStr] = key.split("x");
    const width = Number(wStr);
    const height = Number(hStr);
    try {
      setError(null);
      // Try to keep current refresh rate if possible
      await invoke("set_monitor_resolution", {
        device_name: selected.device_name,
        width,
        height,
        refresh_hz: selected.current.refresh_hz,
      });
      // Refresh monitors after change
      const result = await invoke<DisplayInfo[]>("get_all_monitors");
      setMonitors(result ?? []);
      // Keep selection on same device
      const idx = (result ?? []).findIndex((m) => m.device_name === selected.device_name);
      setSelectedIndex(idx >= 0 ? idx : 0);
      setSelectedResKey(key);
    } catch (e) {
      setError((e as Error).message ?? String(e));
    }
  }

  // Brightness logic moved to component

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
              {`Monitor ${idx + 1}${m.is_primary ? " (Primary)" : ""}`}
            </option>
          ))}
        </select>
      </div>

      {error && <div className="error">{error}</div>}

      <div className="section">
        <label className="label" htmlFor="resolution-select">Resolution</label>
        <ResolutionSelect
          modes={selected?.modes ?? []}
          current={selected?.current ?? null}
          value={selectedResKey}
          disabled={loading || !selected}
          onChange={(next) => {
            setSelectedResKey(next);
            void applyResolution(next);
          }}
        />
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
          </div>
        )}
      </div>

      <BrightnessSlider
        deviceName={selected?.device_name ?? null}
        disabled={loading || !selected}
        onError={(msg) => setError(msg)}
      />
    </div>
  );
}

export default App;
