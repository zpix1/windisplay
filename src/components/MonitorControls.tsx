import BrightnessSlider from "./BrightnessSlider";
import ResolutionSlider from "./ResolutionSlider";
import RefreshRateSlider from "./RefreshRateSlider";
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

      <RefreshRateSlider
        modes={monitor.modes}
        current={monitor.current}
        disabled={disabled}
        deviceName={monitor.device_name}
        onError={onError}
      />
    </>
  );
}
