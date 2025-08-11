import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { DisplayInfo } from "../lib/Resolutions";

type MonitorsContextValue = {
  monitors: DisplayInfo[];
  loading: boolean;
  error: string | null;
  setError: (msg: string | null) => void;
  reloadMonitors: () => Promise<void>;
};

const MonitorsContext = createContext<MonitorsContextValue | undefined>(
  undefined
);

export function MonitorsProvider({ children }: { children: React.ReactNode }) {
  const [monitors, setMonitors] = useState<DisplayInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const reloadMonitors = useCallback(async () => {
    try {
      setLoading(true);
      const result = await invoke<DisplayInfo[]>("get_all_monitors");
      setMonitors(result ?? []);
    } catch (e) {
      setError((e as Error).message ?? String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setLoading(true);
        const result = await invoke<DisplayInfo[]>("get_all_monitors");
        if (!cancelled) setMonitors(result ?? []);
      } catch (e) {
        if (!cancelled) setError((e as Error).message ?? String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const value = useMemo(
    () => ({ monitors, loading, error, setError, reloadMonitors }),
    [monitors, loading, error, reloadMonitors]
  );

  return (
    <MonitorsContext.Provider value={value}>
      {children}
    </MonitorsContext.Provider>
  );
}

export function useMonitorsContext(): MonitorsContextValue {
  const ctx = useContext(MonitorsContext);
  if (!ctx) {
    throw new Error("useMonitorsContext must be used within MonitorsProvider");
  }
  return ctx;
}
