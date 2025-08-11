import "./App.css";
import MonitorControls from "./components/MonitorControls";
import { useMonitorsContext } from "./context/MonitorsContext";
import IdentifyMonitorsButton from "./components/IdentifyMonitorsButton";
import { useEffect, useMemo, useState } from "react";
import { Selector } from "./components/ui/Selector/Selector";

function App() {
  const { monitors, loading, error, setError } = useMonitorsContext();
  const [selectedDeviceName, setSelectedDeviceName] = useState<string | null>(
    null
  );

  // Keep selection valid when monitors list changes
  useEffect(() => {
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
      {error && <div className="error">{error}</div>}

      <div className="sections">
        {loading && (
          <div className="section">
            <div className="muted">Loading...</div>
          </div>
        )}

        {!loading && monitors.length === 0 && (
          <div className="section">
            <div className="muted">No monitors detected</div>
          </div>
        )}

        {monitors.length > 0 && (
          <Selector
            ariaLabel="Select monitor"
            items={monitors}
            selectedItem={selectedMonitor}
            onChange={(m) => setSelectedDeviceName(m.device_name)}
            getKey={(m) => m.device_name}
            getLabel={(m) => monitors.indexOf(m) + 1}
            disabled={loading}
          />
        )}

        {selectedMonitor && (
          <>
            <div className="section-header">{selectedMonitor.display_name}</div>
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
    </div>
  );
}

export default App;
