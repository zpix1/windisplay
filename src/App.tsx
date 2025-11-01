import { debug } from "@tauri-apps/plugin-log";
import { useEffect, useMemo, useState } from "react";
import "./App.css";
import IdentifyMonitorsButton from "./components/IdentifyMonitorsButton";
import MonitorControls from "./components/MonitorControls";
import { Settings } from "./components/Settings/Settings";
import { Selector } from "./components/ui/Selector/Selector";
import { CogIcon } from "./components/ui/icons/CogIcon";
import { MonitorIcon } from "./components/ui/icons/MonitorIcon";
import { useMonitorsContext } from "./context/MonitorsContext";

function App() {
  const { monitors, loading, error, setError } = useMonitorsContext();
  const [selectedDeviceName, setSelectedDeviceName] = useState<string | null>(
    null
  );
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);

  // Keep selection valid when monitors list changes
  useEffect(() => {
    debug(
      `monitors ${JSON.stringify(
        monitors.map((e) => ({
          ...e,
          modes: e.modes.length,
          scales: e.scales.length,
        })),
        null,
        2
      )}`
    );
    if (monitors.length === 0) {
      setSelectedDeviceName(null);
      return;
    }
    const stillExists = monitors.some(
      (m) => m.device_name === selectedDeviceName
    );
    if (!stillExists) {
      setSelectedDeviceName(monitors[0].device_name);
    }
  }, [monitors, selectedDeviceName]);

  const selectedMonitor = useMemo(
    () => monitors.find((m) => m.device_name === selectedDeviceName) ?? null,
    [monitors, selectedDeviceName]
  );

  return (
    <div className="app-root">
      {error && (
        <div className="error">
          {error}{" "}
          <span className="close" onClick={() => setError(null)}>
            close
          </span>
        </div>
      )}

      <div className="sections">
        {!loading && monitors.length === 0 && (
          <div className="section">
            <div className="muted">No monitors detected</div>
          </div>
        )}

        {monitors.length > 1 ? (
          <div className="monitor-selector-container">
            <Selector
              ariaLabel="Select monitor"
              items={monitors}
              selectedItem={selectedMonitor}
              onChange={(m) => setSelectedDeviceName(m.device_name)}
              getKey={(m) => m.device_name}
              getLabel={(m) => monitors.indexOf(m) + 1}
              disabled={loading}
            />
            <button
              className="cog-button"
              onClick={() => setIsSettingsOpen(true)}
              aria-label="Open settings"
              disabled={loading}
            >
              <CogIcon size={20} />
            </button>
          </div>
        ) : (
          <div className="monitor-selector-container single-button">
            <button
              className="cog-button standalone"
              onClick={() => setIsSettingsOpen(true)}
              aria-label="Open settings"
              disabled={loading}
            >
              <CogIcon size={20} />
            </button>
          </div>
        )}

        {selectedMonitor && (
          <>
            <div className="section-header">
              <MonitorIcon
                type={selectedMonitor.built_in ? "laptop" : "external"}
                manufacturer={selectedMonitor.manufacturer}
              />
              <span className="monitor-name">
                <span className="model">
                  {selectedMonitor.model || selectedMonitor.display_name}
                </span>
                <span className="connection">{selectedMonitor.connection}</span>
              </span>
            </div>
            <div className="section">
              <MonitorControls
                monitor={selectedMonitor}
                disabled={loading}
                onError={(msg) => setError(msg)}
              />
            </div>
          </>
        )}

        <div className="section">
          <IdentifyMonitorsButton disabled={loading} onError={setError} />
        </div>
      </div>

      <Settings
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
      />
    </div>
  );
}

export default App;
