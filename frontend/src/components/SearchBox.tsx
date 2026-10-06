import { FormEvent, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { SearchResult } from "../api/types";

export function resultPath(result: SearchResult) {
  const id = encodeURIComponent(result.identifier);
  switch (result.kind) {
    case "transaction":
      return `/txs/${id}`;
    case "account":
      return `/address/${id}`;
    default:
      return `/blocks/${id}`;
  }
}

/**
 * Asks the API what the query is rather than guessing from its shape: a 64-hex value can be a
 * transaction or a block hash. One match goes straight there; anything else opens the results page.
 */
export function SearchBox({
  size = "compact",
  hotkey = false,
}: {
  size?: "compact" | "large";
  /** Focus this box when "/" is pressed outside a text field. One box per page should take it. */
  hotkey?: boolean;
}) {
  const [query, setQuery] = useState("");
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!hotkey) return;
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typing = target?.closest("input, textarea, select, [contenteditable='true']");
      if (event.key === "/" && !typing && !event.metaKey && !event.ctrlKey && !event.altKey) {
        event.preventDefault();
        input.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [hotkey]);
  const [busy, setBusy] = useState(false);
  const navigate = useNavigate();

  const onSubmit = async (event: FormEvent) => {
    event.preventDefault();
    const trimmed = query.trim();
    if (!trimmed || busy) return;
    setBusy(true);
    try {
      const { items } = await explorerApi.search(trimmed);
      if (items.length === 1) {
        navigate(resultPath(items[0]));
        setQuery("");
        return;
      }
    } catch {
      // The results page reports the failure itself.
    } finally {
      setBusy(false);
    }
    navigate(`/search?q=${encodeURIComponent(trimmed)}`);
  };

  return (
    <form className={`search search--${size}`} onSubmit={onSubmit} role="search">
      <svg className="search-icon" viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="7" cy="7" r="4.5" fill="none" stroke="currentColor" strokeWidth="1.6" />
        <path d="M10.5 10.5L14 14" stroke="currentColor" strokeWidth="1.6" />
      </svg>
      <input
        ref={input}
        aria-label="Search"
        aria-keyshortcuts={hotkey ? "/" : undefined}
        placeholder="Search by block, transaction or address"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        spellCheck={false}
        autoComplete="off"
      />
      {hotkey ? (
        <kbd className="search-kbd" aria-hidden="true">
          /
        </kbd>
      ) : null}
      <button type="submit" disabled={busy}>
        {busy ? "Searching…" : "Search"}
      </button>
    </form>
  );
}
