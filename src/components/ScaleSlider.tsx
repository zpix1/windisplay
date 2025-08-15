import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Slider } from "./ui/Slider/Slider";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";
import { ScaleIcon } from "./ui/icons/ScaleIcon";

type Props = {
  deviceName: string | null;
  currentScale: number | null;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

// Common Windows scaling steps (100%..250%)
const allowedScales = [1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5];

export function ScaleSlider({
  deviceName,
  currentScale,
  disabled = false,
  onError,
}: Props) {
  const { mutation } = useMonitorsMutation();

  // Find nearest allowed index to current scale
  const currentIndex = useMemo(() => {
    if (!currentScale || Number.isNaN(currentScale)) return 0;
    let best = 0;
    let bestDiff = Infinity;
    for (let i = 0; i < allowedScales.length; i++) {
      const diff = Math.abs(allowedScales[i] - currentScale);
      if (diff < bestDiff) {
        bestDiff = diff;
        best = i;
      }
    }
    return best;
  }, [currentScale]);

  const [selectedIndex, setSelectedIndex] = useState<number>(currentIndex);
  useEffect(() => setSelectedIndex(currentIndex), [currentIndex]);

  // 0-100 value mapping across discrete steps
  const indexToValue = (idx: number): number => {
    if (allowedScales.length <= 1) return 100;
    return (idx / (allowedScales.length - 1)) * 100;
  };
  const valueToIndex = (val: number): number => {
    if (allowedScales.length <= 1) return 0;
    const n = (val / 100) * (allowedScales.length - 1);
    return Math.round(n);
  };
  const stickyPoints = useMemo(
    () => allowedScales.map((_, i) => indexToValue(i)),
    []
  );

  const applyScale = async (idx: number) => {
    if (!deviceName || !allowedScales[idx]) return;
    const scalePercent = Math.round(allowedScales[idx] * 100);
    try {
      await mutation(() =>
        invoke("set_monitor_scale", { deviceName, scalePercent })
      );
    } catch (e) {
      if (onError) onError((e as Error).message ?? String(e));
    }
  };

  const value = indexToValue(selectedIndex);
  const label = `${Math.round(allowedScales[selectedIndex] * 100)}%`;

  return (
    <div className="field">
      <label className="label" htmlFor="scale-slider">
        Scale
      </label>
      <Slider
        icon={<ScaleIcon />}
        id="scale-slider"
        min={0}
        max={100}
        step={1}
        disabled={disabled}
        value={value}
        onChange={(v) => {
          const idx = valueToIndex(v);
          setSelectedIndex(idx);
        }}
        onValueSubmit={(v) => {
          const idx = valueToIndex(v);
          applyScale(idx);
        }}
        stickyPoints={stickyPoints}
        label={label}
      />
    </div>
  );
}
