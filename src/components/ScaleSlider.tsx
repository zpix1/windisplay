import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Slider } from "./ui/Slider/Slider";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";
import { ScaleIcon } from "./ui/icons/ScaleIcon";
import { ScaleInfo } from "../lib/Resolutions";

type Props = {
  deviceName: string | null;
  currentScale: number | null;
  scales?: ScaleInfo[] | null;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

export function ScaleSlider({
  deviceName,
  currentScale,
  scales,
  disabled = false,
  onError,
}: Props) {
  const { mutation } = useMonitorsMutation();

  // Prefer scales from display data; fall back to common Windows steps
  const availableScales = useMemo(() => {
    const fromDisplay = (scales ?? [])
      .map((s) => s.scale)
      .filter((s) => typeof s === "number" && !Number.isNaN(s));

    // Deduplicate and sort ascending
    const uniqSorted = Array.from(new Set(fromDisplay)).sort((a, b) => a - b);

    // Ensure current scale is represented so the slider can align to it
    const ensureCurrent = () => {
      if (
        currentScale != null &&
        !Number.isNaN(currentScale) &&
        !uniqSorted.includes(currentScale)
      ) {
        return [...uniqSorted, currentScale].sort((a, b) => a - b);
      }
      return uniqSorted;
    };

    const computed = ensureCurrent();

    if (computed.length > 0) return computed;

    // Fallback if the display didn't report scales
    return [1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5];
  }, [scales, currentScale]);

  // Find nearest allowed index to current scale
  const currentIndex = useMemo(() => {
    if (!currentScale || Number.isNaN(currentScale)) return 0;
    let best = 0;
    let bestDiff = Infinity;
    for (let i = 0; i < availableScales.length; i++) {
      const diff = Math.abs(availableScales[i] - currentScale);
      if (diff < bestDiff) {
        bestDiff = diff;
        best = i;
      }
    }
    return best;
  }, [currentScale, availableScales]);

  const [selectedIndex, setSelectedIndex] = useState<number>(currentIndex);
  useEffect(() => setSelectedIndex(currentIndex), [currentIndex]);

  // 0-100 value mapping across discrete steps
  const indexToValue = (idx: number): number => {
    if (availableScales.length <= 1) return 100;
    return (idx / (availableScales.length - 1)) * 100;
  };
  const valueToIndex = (val: number): number => {
    if (availableScales.length <= 1) return 0;
    const n = (val / 100) * (availableScales.length - 1);
    return Math.round(n);
  };
  const stickyPoints = useMemo(
    () => availableScales.map((_, i) => indexToValue(i)),
    [availableScales]
  );

  const applyScale = async (idx: number) => {
    if (!deviceName || !availableScales[idx]) return;
    const scalePercent = Math.round(availableScales[idx] * 100);
    try {
      await mutation(() =>
        invoke("set_monitor_scale", { deviceName, scalePercent })
      );
    } catch (e) {
      if (onError) onError((e as Error).message ?? String(e));
    }
  };

  const value = indexToValue(selectedIndex);
  const label = `${Math.round(availableScales[selectedIndex] * 100)}%`;

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
