import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Slider } from "./ui/Slider/Slider";
import { ResolutionIcon } from "./ui/Slider/icons/ResolutionIcon";
import {
  Resolution,
  PopularResolution,
  getPopularResolutions,
} from "../lib/Resolutions";

type Props = {
  modes: Resolution[];
  current: Resolution | null;
  disabled?: boolean;
  deviceName: string | null;
  onError?: (msg: string) => void;
  onResolutionChanged?: () => void;
};

export default function ResolutionSlider({
  modes,
  current,
  disabled = false,
  deviceName,
  onError,
  onResolutionChanged,
}: Props) {
  const popularResolutions = useMemo(
    () => getPopularResolutions(modes, current),
    [modes, current]
  );

  // Map popular resolutions to sticky points (0-100 scale)
  // Reverse order so 100% = best (highest) resolution
  const stickyPoints = useMemo(() => {
    if (popularResolutions.length === 0) return [];
    return popularResolutions.map(
      (_, index) =>
        ((popularResolutions.length - 1 - index) /
          Math.max(1, popularResolutions.length - 1)) *
        100
    );
  }, [popularResolutions]);

  // Find current resolution index or default to middle
  const currentResIndex = useMemo(() => {
    if (!current) return Math.floor(popularResolutions.length / 2);
    const currentKey = `${current.width}x${current.height}`;
    const index = popularResolutions.findIndex((res) => res.key === currentKey);
    return index >= 0 ? index : Math.floor(popularResolutions.length / 2);
  }, [current, popularResolutions]);

  const [selectedIndex, setSelectedIndex] = useState<number>(currentResIndex);

  // Update selectedIndex when current resolution changes
  useEffect(() => {
    setSelectedIndex(currentResIndex);
  }, [currentResIndex]);

  // Apply resolution change
  const applyResolution = async (resolutionIndex: number) => {
    if (!deviceName || !popularResolutions[resolutionIndex]) return;

    const resolution = popularResolutions[resolutionIndex];
    try {
      await invoke("set_monitor_resolution", {
        deviceName,
        width: resolution.width,
        height: resolution.height,
        refresh_hz: current?.refresh_hz || 60,
      });

      if (onResolutionChanged) {
        onResolutionChanged();
      }
    } catch (e) {
      if (onError) {
        onError((e as Error).message ?? String(e));
      }
    }
  };

  // Convert slider value (0-100) to resolution index (reversed so 100% = best resolution)
  const valueToIndex = (sliderValue: number): number => {
    if (popularResolutions.length === 0) return 0;
    const normalizedIndex =
      (sliderValue / 100) * Math.max(0, popularResolutions.length - 1);
    return popularResolutions.length - 1 - Math.round(normalizedIndex);
  };

  // Convert resolution index to slider value (0-100) (reversed so 100% = best resolution)
  const indexToValue = (index: number): number => {
    if (popularResolutions.length === 0) return 50;
    const reversedIndex = popularResolutions.length - 1 - index;
    return (reversedIndex / Math.max(1, popularResolutions.length - 1)) * 100;
  };

  const currentValue = indexToValue(selectedIndex);

  // Get current resolution display text
  const currentResolution = popularResolutions[selectedIndex];
  const displayText = currentResolution ? currentResolution.text : "Resolution";

  return (
    <div className="section">
      <label className="label" htmlFor="resolution-slider">
        Resolution
      </label>
      <Slider
        id="resolution-slider"
        min={0}
        max={100}
        step={1}
        disabled={disabled || popularResolutions.length === 0}
        value={currentValue}
        onChange={(newValue) => {
          const newIndex = valueToIndex(newValue);
          setSelectedIndex(newIndex);
        }}
        onValueSubmit={(newValue) => {
          const newIndex = valueToIndex(newValue);
          applyResolution(newIndex);
        }}
        stickyPoints={stickyPoints}
        label={displayText}
        icon={<ResolutionIcon />}
      />
    </div>
  );
}
