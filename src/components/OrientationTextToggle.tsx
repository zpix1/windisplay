import { useMemo, useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";
import {
  PagedSelect,
  type PagedSelectItem,
} from "./ui/PagedSelect/PagedSelect";
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
      { key: "0", degrees: 0, label: "Landscape" },
      { key: "90", degrees: 90, label: "Portrait" },
      { key: "180", degrees: 180, label: "Landscape (flipped)" },
      { key: "270", degrees: 270, label: "Portrait (flipped)" },
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

  const buildItemsForPage = (page: number): ReadonlyArray<PagedSelectItem> => {
    if (page !== 1) return [];
    return options.map((opt) => ({
      key: opt.key,
      label: opt.label,
      icon: (
        <DisplayOrientationIcon
          aspectRatioKey={aspectKey}
          orientation={opt.degrees}
        />
      ),
    }));
  };

  const triggerIcon = (
    <DisplayOrientationIcon
      aspectRatioKey={aspectKey}
      orientation={selected?.degrees ?? orientation ?? 0}
    />
  );

  const handleSelect = async (key: string, label: string) => {
    const opt = options.find((o) => o.key === key);
    if (!opt) return;
    setSelected(opt);
    await apply(opt);
  };

  return (
    <>
      <div className="field">
        <div className="label">Orientation</div>
        <PagedSelect
          disabled={disabled}
          triggerLabel={selected?.label ?? "Orientation"}
          triggerIcon={triggerIcon}
          pageCount={1}
          getItemsForPage={buildItemsForPage}
          selectedLabel={selected?.label ?? undefined}
          onSelect={handleSelect}
        />
      </div>
    </>
  );
}
