import { useMemo, useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";
import { TextToggle } from "./ui/TextToggle/TextToggle";
import { DisplayOrientationIcon } from "./ui/icons/DisplayOrientationIcon";

type OrientationOption = {
  key: string;
  degrees: number;
  label: string;
};

type Props = {
  deviceName: string | null;
  orientation: number | null; // 0,90,180,270
  disabled?: boolean;
  onError?: (msg: string) => void;
  // Optional: aspect ratio string (e.g. "16:9"). If not provided, computed from current.
  aspectRatioKey?: string;
};

export function OrientationSelector({
  deviceName,
  orientation,
  disabled = false,
  onError,
  aspectRatioKey,
}: Props) {
  const { mutation } = useMonitorsMutation();

  const options: OrientationOption[] = useMemo(
    () => [
      { key: "0", degrees: 0, label: "Landscape orientation" },
      { key: "90", degrees: 90, label: "Portrait orientation" },
      { key: "180", degrees: 180, label: "Landscape orientation (flipped)" },
      { key: "270", degrees: 270, label: "Portrait orientation (flipped)" },
    ],
    []
  );

  const currentOption = useMemo(() => {
    const deg = (orientation ?? 0) % 360;
    return options.find((o) => o.degrees === deg) ?? options[0];
  }, [orientation, options]);

  const [selected, setSelected] = useState<OrientationOption | null>(
    currentOption
  );
  useEffect(() => setSelected(currentOption), [currentOption]);

  const apply = async (opt: OrientationOption) => {
    if (!deviceName) return;
    try {
      await mutation(() =>
        invoke("set_monitor_orientation", {
          deviceName,
          orientationDegrees: opt.degrees,
        })
      );
    } catch (e) {
      if (onError) onError((e as Error).message ?? String(e));
    }
  };

  const aspectKey = useMemo(() => {
    if (aspectRatioKey && /\d+\s*:\s*\d+/.test(aspectRatioKey))
      return aspectRatioKey;
    return "16:9";
  }, [aspectRatioKey]);

  console.log("aspectKey", aspectKey);

  return (
    <>
      <TextToggle
        hideIconBackground
        toggled={false}
        text={selected?.label ?? "Orientation"}
        disabled={disabled}
        icon={
          <DisplayOrientationIcon
            aspectRatioKey={aspectKey}
            orientation={orientation ?? 0}
          />
        }
        onClick={() => {
          // Future: open a dialog or cycle through options.
          // For now, simply cycle to the next option on click.
          const idx = options.findIndex((o) => o.key === selected?.key);
          const next = options[(idx + 1) % options.length];
          setSelected(next);
          apply(next);
        }}
      />
    </>
  );
}
