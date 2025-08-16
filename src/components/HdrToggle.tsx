import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DisplayInfo } from "../lib/Resolutions";
import { HdrIcon } from "./ui/icons/HdrIcon";
import { TextToggle } from "./ui/TextToggle/TextToggle";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";

type HdrToggleProps = {
  monitor: DisplayInfo;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

export function HdrToggle({ monitor, disabled, onError }: HdrToggleProps) {
  const [busy, setBusy] = useState(false);
  const { mutation } = useMonitorsMutation();
  const label = {
    on: "HDR Enabled",
    off: "HDR Disabled",
    unsupported: "HDR Unsupported",
  }[monitor.hdr_status];

  return (
    <TextToggle
      toggled={monitor.hdr_status === "on"}
      text={label}
      disabled={disabled || busy}
      onClick={async () => {
        if (busy) return;
        try {
          setBusy(true);
          const enable = monitor.hdr_status !== "on";
          await mutation(() =>
            invoke("enable_hdr", {
              deviceName: monitor.device_name,
              enable,
            })
          );
        } catch (e) {
          // Error already surfaced via MonitorsContext.mutation
          if (onError) onError((e as Error).message ?? String(e));
        } finally {
          setBusy(false);
        }
      }}
      icon={<HdrIcon on={monitor.hdr_status === "on"} />}
      hideIconBackground
    />
  );
}
