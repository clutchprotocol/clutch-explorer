import { FormEvent, useState } from "react";
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
export function SearchBox({ size = "compact" }: { size?: "compact" | "large" }) {
  const [query, setQuery] = useState("");
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
        aria-label="Search"
        placeholder="Search by block, transaction or address"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        spellCheck={false}
        autoComplete="off"
      />
      <button type="submit" disabled={busy}>
        {busy ? "Searching…" : "Search"}
      </button>
    </form>
  );
}
