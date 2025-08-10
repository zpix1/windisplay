import { useCallback, useRef } from "react";

export function useThrottle<T extends (...args: any[]) => any>(
  callback: T,
  delay: number
): T {
  const lastCallTime = useRef<number>(0);
  const timeoutRef = useRef<number | null>(null);

  return useCallback(
    ((...args: Parameters<T>) => {
      const now = Date.now();

      // Clear any pending timeout
      if (timeoutRef.current) {
        window.clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }

      // If enough time has passed since last call, execute immediately
      if (now - lastCallTime.current >= delay) {
        lastCallTime.current = now;
        return callback(...args);
      }

      // Otherwise, schedule execution for later
      timeoutRef.current = window.setTimeout(() => {
        lastCallTime.current = Date.now();
        callback(...args);
        timeoutRef.current = null;
      }, delay - (now - lastCallTime.current));
    }) as T,
    [callback, delay]
  );
}
