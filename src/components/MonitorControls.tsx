import { BrightnessSlider } from "./BrightnessSlider";
import { ResolutionSlider } from "./ResolutionSlider";
import { RefreshRateSlider } from "./RefreshRateSlider";
import { aspectKey, DisplayInfo } from "../lib/Resolutions";
import { OrientationSelector } from "./OrientationTextToggle";
import { ScaleSlider } from "./ScaleSlider.tsx";

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
        orientationDegrees={monitor.orientation}
        maxNative={monitor.max_native}
      />

      <RefreshRateSlider
        modes={monitor.modes}
        current={monitor.current}
        disabled={disabled}
        deviceName={monitor.device_name}
        onError={onError}
      />

      <ScaleSlider
        deviceName={monitor.device_name}
        currentScale={monitor.scale}
        scales={monitor.scales}
        disabled={disabled}
        onError={onError}
      />

      <OrientationSelector
        deviceName={monitor.device_name}
        orientation={monitor.orientation}
        aspectRatioKey={aspectKey(
          monitor.current.width,
          monitor.current.height
        )}
        disabled={disabled}
        onError={onError}
      />
    </>
  );
}
