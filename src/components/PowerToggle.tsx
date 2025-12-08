import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DisplayInfo } from "../lib/Resolutions";
import { PowerIcon } from "./ui/icons/PowerIcon";
import { TextToggle } from "./ui/TextToggle/TextToggle";
import { useMonitorsMutation } from "../hooks/useMonitorsMutation";

type PowerToggleProps = {
  monitor: DisplayInfo;
  disabled?: boolean;
  onError?: (msg: string) => void;
};

export function PowerToggle({ monitor, disabled, onError }: PowerToggleProps) {
  const [busy, setBusy] = useState(false);
  const { mutation } = useMonitorsMutation();
  const isOn = monitor.enabled;

  return (
    <TextToggle
      toggled={isOn}
      loading={busy}
      text={isOn ? "Power On" : "Power Off"}
      disabled={disabled || busy}
      onClick={async () => {
        if (busy) return;
        try {
          setBusy(true);
          const powerOn = !isOn;
          await mutation(() =>
            invoke("set_monitor_power", {
              deviceName: monitor.device_name,
              powerOn,
            })
          );
        } catch (e) {
          if (onError) onError((e as Error).message ?? String(e));
        } finally {
          setBusy(false);
        }
      }}
      icon={<PowerIcon />}
    />
  );
}
