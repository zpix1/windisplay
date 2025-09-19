import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";
import {
  DisplayPortIcon,
  DviIcon,
  HdmiIcon,
  SVideoIcon,
  TunerIcon,
  UsbCIcon,
  VgaIcon,
  CompositeIcon,
} from "./ui/icons/InputIcons";
import { PagedSelect, type PagedSelectItem } from "./ui/PagedSelect/PagedSelect";

type InputSourceSelectorProps = {
  deviceName: string;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

type InputOption = {
  key: string;
  label: string;
};

export function InputSourceSelector({
  deviceName,
  disabled = false,
  onError,
}: Readonly<InputSourceSelectorProps>) {
  const { mutation } = useMonitorsMutation();
  const [loadingKey, setLoadingKey] = useState<string | null>(null);
  const [selectedLabel, setSelectedLabel] = useState<string>("Select input");

  // Fetch current active input to highlight the item
  useEffect(() => {
    let mounted = true;
    (async () => {
      try {
        const current = await invoke<string>("get_monitor_input_source", {
          deviceName,
        });
        if (mounted && current) {
          // Map code/label to human label best-effort
          const map: Record<string, string> = {
            dp1: "DisplayPort 1",
            dp2: "DisplayPort 2",
            hdmi1: "HDMI 1",
            hdmi2: "HDMI 2",
            hdmi3: "HDMI 3",
            dvi1: "DVI 1",
            dvi2: "DVI 2",
            vga1: "VGA 1",
            vga2: "VGA 2",
          };
          const label = map[current.toLowerCase()] ?? current;
          setSelectedLabel(label);
        }
      } catch {
        // ignore if unsupported
      }
    })();
    return () => {
      mounted = false;
    };
  }, [deviceName]);

  // Resolve icon for a given label
  const iconForLabel = (label: string) => {
    const l = label.toLowerCase();
    if (l.includes("displayport") || l.startsWith("dp"))
      return <DisplayPortIcon />;
    if (l.includes("hdmi")) return <HdmiIcon />;
    if (l.includes("dvi")) return <DviIcon />;
    if (l.includes("usb-c") || l.includes("tb") || l.includes("usbc"))
      return <UsbCIcon />;
    if (l.includes("composite")) return <CompositeIcon />;
    if (l.includes("s-video") || l.includes("svideo")) return <SVideoIcon />;
    if (l.includes("tuner")) return <TunerIcon />;
    if (l.includes("vga")) return <VgaIcon />;
    return <DisplayPortIcon />;
  };

  // Build items for PagedSelect from a page index
  const buildItemsForPage = (page: number): ReadonlyArray<PagedSelectItem> => {
    const opts = getItemsForPage(page);
    return opts.map<PagedSelectItem>((opt) => ({
      key: opt.key,
      label: opt.label,
      icon: iconForLabel(opt.label),
    }));
  };

  // Trigger icon inferred from current selection
  const triggerIcon = iconForLabel(selectedLabel);

  // Handler for selection
  const handleSelect = async (key: string, label: string) => {
    if (loadingKey) return;
    setLoadingKey(key);
    try {
      await mutation(async () => {
        await invoke("set_monitor_input_source", { deviceName, input: key });
      });
      setSelectedLabel(label);
    } catch (e) {
      onError?.((e as Error).message ?? String(e));
    } finally {
      setLoadingKey(null);
    }
  };

  const PAGE_1: ReadonlyArray<InputOption> = [
    { key: "dp1", label: "DisplayPort 1" },
    { key: "dp2", label: "DisplayPort 2" },
    { key: "hdmi1", label: "HDMI 1" },
    { key: "hdmi2", label: "HDMI 2" },
    { key: "hdmi3", label: "HDMI 3" },
    { key: "usbc1", label: "USB-C / TB 1" },
    { key: "usbc2", label: "USB-C / TB 2" },
    { key: "usbc3", label: "USB-C / TB 3" },
    { key: "usbc4", label: "USB-C / TB 4" },
  ];
  const PAGE_2: ReadonlyArray<InputOption> = [
    { key: "dvi1", label: "DVI 1" },
    { key: "dvi2", label: "DVI 2" },
    { key: "vga1", label: "VGA 1" },
    { key: "vga2", label: "VGA 2" },
  ];
  const PAGE_3: ReadonlyArray<InputOption> = [
    { key: "dp1_lg", label: "DisplayPort 1 (LG alt)" },
    { key: "dp2_usbc_lg", label: "DP 2 / USB-C (LG alt)" },
    { key: "usbc_lg", label: "USB-C (LG alt)" },
    { key: "hdmi1_lg", label: "HDMI 1 (LG alt)" },
    { key: "hdmi2_lg", label: "HDMI 2 (LG alt)" },
  ];
  const PAGE_4: ReadonlyArray<InputOption> = [
    { key: "composite1", label: "Composite 1 (Legacy)" },
    { key: "composite2", label: "Composite 2 (Legacy)" },
    { key: "svideo1", label: "S-Video 1 (Legacy)" },
    { key: "svideo2", label: "S-Video 2 (Legacy)" },
    { key: "tuner1", label: "Tuner 1 (Legacy)" },
    { key: "tuner2", label: "Tuner 2 (Legacy)" },
    { key: "tuner3", label: "Tuner 3 (Legacy)" },
    { key: "component1", label: "Component 1 (Legacy)" },
    { key: "component2", label: "Component 2 (Legacy)" },
    { key: "component3", label: "Component 3 (Legacy)" },
  ];

  const getItemsForPage = (p: number): ReadonlyArray<InputOption> => {
    switch (p) {
      case 1:
        return PAGE_1;
      case 2:
        return PAGE_2;
      case 3:
        return PAGE_3;
      case 4:
        return PAGE_4;
      default:
        return PAGE_1;
    }
  };

  return (
    <div className="field">
      <div className="label">Input source</div>
      <PagedSelect
        disabled={disabled}
        triggerLabel={loadingKey ? "Switching..." : selectedLabel}
        triggerIcon={triggerIcon}
        pageCount={4}
        getItemsForPage={buildItemsForPage}
        selectedLabel={selectedLabel}
        onSelect={handleSelect}
      />
    </div>
  );
}
