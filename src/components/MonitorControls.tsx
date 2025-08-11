import BrightnessSlider from "./BrightnessSlider";
import ResolutionSlider from "./ResolutionSlider";
import { DisplayInfo } from "../lib/Resolutions";

type MonitorControlsProps = {
  monitor: DisplayInfo;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

export default function MonitorControls({
  monitor,
  disabled = false,
  onError,
}: MonitorControlsProps) {
  return (
    <>
      <div className="section">
        <div className="details">
          <div className="row">
            <span className="key">Resolution:</span>
            <span className="value">
              {monitor.current.width} × {monitor.current.height}
            </span>
          </div>
          <div className="row">
            <span className="key">Refresh rate:</span>
            <span className="value">{monitor.current.refresh_hz} Hz</span>
          </div>
        </div>
      </div>

      <BrightnessSlider
        deviceName={monitor.device_name}
        disabled={disabled}
        onError={onError}
      />

      <ResolutionSlider
        modes={monitor.modes}
        current={monitor.current}
        disabled={disabled}
        deviceName={monitor.device_name}
      />
    </>
  );
}
