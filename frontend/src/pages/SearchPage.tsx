import { Link, useSearchParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import { resultPath, SearchBox } from "../components/SearchBox";
import { EmptyState, ErrorBanner, PageHeader, Panel, Skeleton } from "../components/ui";
import { useApi } from "../hooks/useApi";

const KIND_LABEL = { block: "Block", transaction: "Transaction", account: "Account" } as const;

export function SearchPage() {
  const [params] = useSearchParams();
  const q = (params.get("q") ?? "").trim();
  const { data, error } = useApi(() => (q ? explorerApi.search(q) : Promise.resolve({ items: [] })), [q]);

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Search" title={q ? <>Results for <span className="mono">{q}</span></> : "Search"} />
      <ErrorBanner message={error ? `Search failed: ${error.message}` : undefined} />
      <Panel>
        {!data ? (
          error ? null : <Skeleton rows={3} />
        ) : data.items.length === 0 ? (
          <EmptyState>
            Nothing on this chain matches that. Search takes a block height, a block or transaction
            hash (with or without 0x), or a 0x account address.
          </EmptyState>
        ) : (
          <ul className="results">
            {data.items.map((item) => (
              <li key={`${item.kind}-${item.identifier}`}>
                <span className="type-badge type-badge--neutral">{KIND_LABEL[item.kind] ?? item.kind}</span>
                <Link className="mono" to={resultPath(item)}>
                  {item.identifier}
                </Link>
                <span className="muted small">{item.summary}</span>
              </li>
            ))}
          </ul>
        )}
      </Panel>
      <SearchBox size="large" />
    </div>
  );
}
