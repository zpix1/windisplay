import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Slider } from "./ui/Slider/Slider";
import { Resolution } from "../lib/Resolutions";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";
import { RefreshRateIcon } from "./ui/icons/RefreshRateIcon";

type Props = {
  modes: Resolution[];
  current: Resolution | null;
  disabled?: boolean;
  deviceName: string | null;
  onError?: (msg: string) => void;
};

export function RefreshRateSlider({
  modes,
  current,
  disabled = false,
  deviceName,
  onError,
}: Props) {
  const { mutation } = useMonitorsMutation();

  // Available refresh rates for the current resolution (width x height)
  const availableHz = useMemo(() => {
    if (!current) return [] as number[];
    const set = new Set<number>();
    for (const m of modes) {
      if (m.width === current.width && m.height === current.height) {
        set.add(m.refresh_hz);
      }
    }
    return Array.from(set).sort((a, b) => a - b);
  }, [modes, current]);

  // Index of the current refresh rate within availableHz
  const currentHzIndex = useMemo(() => {
    if (!current || availableHz.length === 0) return 0;
    const idx = availableHz.findIndex((hz) => hz === current.refresh_hz);
    return idx >= 0 ? idx : 0;
  }, [availableHz, current]);

  const [selectedIndex, setSelectedIndex] = useState<number>(currentHzIndex);

  useEffect(() => {
    setSelectedIndex(currentHzIndex);
  }, [currentHzIndex]);

  // Map indices to a 0-100 slider scale (100% = highest Hz)
  const indexToValue = (index: number): number => {
    if (availableHz.length <= 1) return 100;
    return (index / (availableHz.length - 1)) * 100;
  };

  const valueToIndex = (value: number): number => {
    if (availableHz.length <= 1) return 0;
    const normalizedIndex = (value / 100) * (availableHz.length - 1);
    return Math.round(normalizedIndex);
  };

  // Sticky points at each discrete refresh option
  const stickyPoints = useMemo(() => {
    if (availableHz.length === 0) return [] as number[];
    return availableHz.map((_, i) => indexToValue(i));
  }, [availableHz]);

  const applyRefreshRate = async (index: number) => {
    if (!deviceName || !current || !availableHz[index]) return;
    const nextHz = availableHz[index];
    try {
      await mutation(() =>
        invoke("set_monitor_resolution", {
          deviceName,
          width: current.width,
          height: current.height,
          refreshHz: nextHz,
        })
      );
    } catch (e) {
      if (onError) onError((e as Error).message ?? String(e));
    }
  };

  const value = indexToValue(selectedIndex);
  const label =
    availableHz.length > 0
      ? `${availableHz[selectedIndex]} Hz`
      : current
      ? `${current.refresh_hz} Hz`
      : "Refresh rate";

  return (
    <div className="field">
      <label className="label" htmlFor="refresh-rate-slider">
        Refresh rate
      </label>
      <Slider
        id="refresh-rate-slider"
        min={0}
        max={100}
        step={1}
        disabled={disabled || !current || availableHz.length === 0}
        value={value}
        onChange={(v) => {
          const idx = valueToIndex(v);
          setSelectedIndex(idx);
        }}
        onValueSubmit={(v) => {
          const idx = valueToIndex(v);
          applyRefreshRate(idx);
        }}
        stickyPoints={stickyPoints}
        label={label}
        icon={<RefreshRateIcon />}
      />
    </div>
  );
}
