import { useSearchParams } from "react-router-dom";

/** The 1-based page number kept in `?page=`, so Back and shared links land on the same page. */
export function usePage(): [number, (page: number) => void] {
  const [params, setParams] = useSearchParams();
  const raw = Number.parseInt(params.get("page") ?? "1", 10);
  const page = Number.isFinite(raw) && raw > 0 ? raw : 1;
  const setPage = (next: number) => {
    const updated = new URLSearchParams(params);
    if (next <= 1) updated.delete("page");
    else updated.set("page", String(next));
    setParams(updated);
    window.scrollTo({ top: 0 });
  };
  return [page, setPage];
}
