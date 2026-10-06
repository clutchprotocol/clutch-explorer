import { useSearchParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import { TransactionsTable } from "../components/tables";
import { ErrorBanner, PageHeader, Pagination, Panel, Skeleton } from "../components/ui";
import { useApi } from "../hooks/useApi";
import { usePage } from "../hooks/usePage";

const PAGE_SIZE = 25;
const STATUSES = ["", "confirmed", "pending", "failed"];

export function TransactionsPage() {
  const [page, setPage] = usePage();
  const [params, setParams] = useSearchParams();
  const status = params.get("status") ?? "";

  const { data, error } = useApi(
    () => explorerApi.getTransactions(PAGE_SIZE, (page - 1) * PAGE_SIZE, { status: status || undefined }),
    [page, status],
  );

  const setStatus = (next: string) => {
    const updated = new URLSearchParams();
    if (next) updated.set("status", next);
    setParams(updated);
  };

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Chain" title="Transactions">
        <p className="lede">Transfers, ride steps, mints and burns, newest first.</p>
      </PageHeader>
      <ErrorBanner message={error?.message} />
      <Panel
        flush
        actions={
          <div className="segmented" role="group" aria-label="Filter by status">
            {STATUSES.map((value) => (
              <button
                key={value || "all"}
                type="button"
                aria-pressed={status === value}
                className={status === value ? "is-active" : ""}
                onClick={() => setStatus(value)}
              >
                {value || "All"}
              </button>
            ))}
          </div>
        }
      >
        {data ? (
          <TransactionsTable
            transactions={data.items}
            empty={status ? `No ${status} transactions.` : "No transactions indexed yet."}
          />
        ) : error ? null : (
          <Skeleton rows={12} />
        )}
        <Pagination page={page} hasMore={Boolean(data?.paging.has_more)} onChange={setPage} />
      </Panel>
    </div>
  );
}
