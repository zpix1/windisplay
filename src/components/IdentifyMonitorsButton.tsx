import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SearchIcon } from "./ui/icons/SearchIcon";
import { TextToggle } from "./ui/TextToggle/TextToggle";

type IdentifyMonitorsButtonProps = {
  disabled?: boolean;
  onError?: (msg: string) => void;
};

export default function IdentifyMonitorsButton({
  disabled = false,
  onError,
}: IdentifyMonitorsButtonProps) {
  const [loading, setLoading] = useState(false);

  const handleClick = useCallback(async () => {
    try {
      setLoading(true);
      await invoke("identify_monitors");
    } catch (e) {
      if (onError) onError((e as Error).message ?? String(e));
    } finally {
      setLoading(false);
    }
  }, [onError]);

  return (
    <TextToggle
      toggled={loading}
      text={"Identify Monitors"}
      disabled={loading || disabled}
      icon={<SearchIcon />}
      onClick={handleClick}
    />
  );
}
