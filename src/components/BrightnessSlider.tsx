import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useThrottle } from "../hooks/useDebouncedCallback";
import { Slider } from "./ui/Slider/Slider";
import { BrightnessIcon } from "./ui/icons/BrightnessIcon";
import { useMonitorsContext } from "../context/MonitorsContext";

type Props = {
  deviceName: string | null;
  requiresWmi?: boolean;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

export function BrightnessSlider({
  deviceName,
  requiresWmi,
  disabled,
  onError,
}: Props) {
  const [pct, setPct] = useState<number | null>(null);
  const { monitors } = useMonitorsContext();

  const fetchBrightness = useCallback(async () => {
    if (!deviceName) {
      setPct(null);
      return;
    }
    try {
      const info = await invoke<{
        min: number;
        current: number;
        max: number;
      }>("get_monitor_brightness", { deviceName });
      if (info) {
        const span = Math.max(1, info.max - info.min);
        const val = Math.round(((info.current - info.min) / span) * 100);
        setPct(Math.max(0, Math.min(100, val)));
      }
    } catch (e) {
      if (onError) onError((e as Error).message ?? String(e));
    }
  }, [deviceName, onError]);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      if (cancelled) return;
      await fetchBrightness();
    })();
    return () => {
      cancelled = true;
    };
  }, [fetchBrightness, monitors]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    (async () => {
      unlisten = await listen("brightness-changed", async () => {
        await fetchBrightness();
      });
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, [fetchBrightness]);

  const apply = useCallback(
    async (next: number) => {
      if (!deviceName) return;
      try {
        await invoke("set_monitor_brightness", { deviceName, percent: next });
      } catch (e) {
        console.error(e);
      }
    },
    [deviceName]
  );

  const throttledApply = useThrottle(apply, requiresWmi ? 500 : 100);

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
        disabled={disabled || pct === null}
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
