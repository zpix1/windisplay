import { useCallback } from "react";
import { useMonitorsContext } from "../context/MonitorsContext";

export function useMonitorsMutation() {
  const { reloadMonitors, setError } = useMonitorsContext();

  const mutation = useCallback(
    async <T>(action: () => Promise<T>): Promise<T> => {
      try {
        const result = await action();
        await reloadMonitors();
        return result;
      } catch (e) {
        const msg = (e as Error).message ?? String(e);
        setError(msg);
        throw e;
      }
    },
    [reloadMonitors, setError]
  );

  return { mutation } as const;
}
