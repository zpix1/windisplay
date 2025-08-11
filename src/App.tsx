import "./App.css";
import MonitorControls from "./components/MonitorControls";
import { useMonitorsContext } from "./context/MonitorsContext";
import IdentifyMonitorsButton from "./components/IdentifyMonitorsButton";

function App() {
  const { monitors, loading, error, setError } = useMonitorsContext();

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
        {monitors.map((monitor) => (
          <div key={monitor.device_name}>
            <div className="section-header">{monitor.display_name}</div>
            <div className="section">
              <MonitorControls
                monitor={monitor}
                disabled={loading}
                onError={(msg) => setError(msg)}
              />
            </div>
          </div>
        ))}
        <div className="section">
          <IdentifyMonitorsButton disabled={loading} onError={setError} />
        </div>
      </div>
    </div>
  );
}

export default App;
