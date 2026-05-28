import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { DisplayInfo } from "../lib/Resolutions";

type MonitorsContextValue = {
  monitors: DisplayInfo[];
  loading: boolean;
  refreshing: boolean;
  error: string | null;
  setError: (msg: string | null) => void;
  reloadMonitors: () => Promise<void>;
};

const MonitorsContext = createContext<MonitorsContextValue | undefined>(
  undefined
);

export function MonitorsProvider({ children }: { children: React.ReactNode }) {
  const [monitors, setMonitors] = useState<DisplayInfo[]>([]);
  const [initialLoading, setInitialLoading] = useState<boolean>(true);
  const [refreshing, setRefreshing] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const reloadPromiseRef = useRef<Promise<void> | null>(null);

  const reloadMonitors = useCallback(async () => {
    if (reloadPromiseRef.current) {
      return reloadPromiseRef.current;
    }

    const promise = (async () => {
      try {
        setRefreshing(true);
        const result = await invoke<DisplayInfo[]>("get_all_monitors");
        const displayNameInfo = result?.map((m, idx) => ({
          ...m,
          display_name: `Monitor ${idx + 1}${m.is_primary ? " (Primary)" : ""}`,
        }));
        setMonitors(displayNameInfo ?? []);
      } catch (e) {
        setError((e as Error).message ?? String(e));
      } finally {
        setRefreshing(false);
        setInitialLoading(false);
        reloadPromiseRef.current = null;
      }
    })();

    reloadPromiseRef.current = promise;
    return promise;
  }, []);

  useEffect(() => {
    reloadMonitors();
  }, []);

  // Listen for display change events from the backend (Windows system events)
  useEffect(() => {
    const unlisten = listen("display-changed", () => {
      reloadMonitors();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [reloadMonitors]);

  const value = useMemo(
    () => ({
      monitors,
      loading: initialLoading,
      refreshing,
      error,
      setError,
      reloadMonitors,
    }),
    [monitors, initialLoading, refreshing, error, reloadMonitors]
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
