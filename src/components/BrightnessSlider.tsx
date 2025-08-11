import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useThrottle } from "../hooks/useDebouncedCallback";
import { Slider } from "./ui/Slider/Slider";
import { BrightnessIcon } from "./ui/Slider/icons/BrightnessIcon";

type Props = {
  deviceName: string | null;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

export default function BrightnessSlider({
  deviceName,
  disabled,
  onError,
}: Props) {
  const [pct, setPct] = useState<number | null>(null);
  const [loading, setLoading] = useState<boolean>(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      if (!deviceName) {
        setPct(null);
        return;
      }
      try {
        setLoading(true);
        const info = await invoke<{
          min: number;
          current: number;
          max: number;
        }>("get_monitor_brightness", { deviceName });
        if (!cancelled && info) {
          const span = Math.max(1, info.max - info.min);
          const val = Math.round(((info.current - info.min) / span) * 100);
          setPct(Math.max(0, Math.min(100, val)));
        }
      } catch (e) {
        if (!cancelled && onError) onError((e as Error).message ?? String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [deviceName]);

  const apply = useCallback(
    async (next: number) => {
      console.log("apply", deviceName, next);

      if (!deviceName) return;
      try {
        await invoke("set_monitor_brightness", { deviceName, percent: next });
      } catch (e) {
        if (onError) onError((e as Error).message ?? String(e));
      }
    },
    [deviceName, onError]
  );

  const throttledApply = useThrottle(apply, 50);

  return (
    <div className="field">
      <label className="label" htmlFor="brightness-range">
        Brightness
      </label>
      <Slider
        id="brightness-range"
        min={0}
        max={100}
        step={1}
        disabled={disabled || loading || pct === null}
        value={pct ?? 0}
        onChange={(next) => {
          setPct(next);
          console.log("onChange", deviceName, next);
          throttledApply(next);
        }}
        icon={<BrightnessIcon />}
        label={`${pct ?? 0}%`}
      />
    </div>
  );
}
