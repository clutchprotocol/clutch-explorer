import { DependencyList, useEffect, useState } from "react";
import { ApiError } from "../api/client";

export type ApiState<T> = {
  data: T | undefined;
  error: ApiError | undefined;
  loading: boolean;
  /** When the last successful load finished, for a "live" indicator. */
  updatedAt: number | undefined;
};

function toApiError(err: unknown) {
  return err instanceof ApiError ? err : new ApiError((err as Error)?.message ?? "Unknown error", 0);
}

/**
 * Loads once per change of `deps`, and again every `pollMs` if given. A failed poll keeps the data
 * already on screen and only sets `error`, so a blip does not blank the page.
 */
export function useApi<T>(load: () => Promise<T>, deps: DependencyList, pollMs?: number) {
  const [state, setState] = useState<ApiState<T>>({
    data: undefined,
    error: undefined,
    loading: true,
    updatedAt: undefined,
  });

  useEffect(() => {
    let disposed = false;
    setState({ data: undefined, error: undefined, loading: true, updatedAt: undefined });

    const run = async () => {
      try {
        const data = await load();
        if (!disposed) setState({ data, error: undefined, loading: false, updatedAt: Date.now() });
      } catch (err) {
        if (!disposed) setState((prev) => ({ ...prev, error: toApiError(err), loading: false }));
      }
    };

    run();
    const id = pollMs ? window.setInterval(run, pollMs) : undefined;
    return () => {
      disposed = true;
      if (id) window.clearInterval(id);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  return state;
}
