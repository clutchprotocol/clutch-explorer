import { useSearchParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import { TransactionsTable } from "../components/tables";
import { ErrorBanner, PageHeader, Pagination, Panel, Skeleton } from "../components/ui";
import { useApi } from "../hooks/useApi";
import { usePage } from "../hooks/usePage";

const PAGE_SIZE = 25;

/** The transaction types a reader would filter by, grouped the way the chain uses them. */
const TYPE_GROUPS: { label: string; types: string[] }[] = [
  { label: "Payments", types: ["Transfer"] },
  {
    label: "Rides",
    types: ["RideRequest", "RideOffer", "RideAcceptance", "RidePay", "RideCancel", "RideRequestCancel"],
  },
  { label: "Treasury", types: ["Mint", "Burn"] },
];

const spaced = (type: string) => type.replace(/([a-z])([A-Z])/g, "$1 $2");

export function TransactionsPage() {
  const [page, setPage] = usePage();
  const [params, setParams] = useSearchParams();
  // `status` is honoured if a link carries it, but not offered: the indexer stores "confirmed"
  // for every transaction today, so a status picker could not narrow anything.
  const status = params.get("status") ?? "";
  const type = params.get("type") ?? "";

  const { data, error } = useApi(
    () =>
      explorerApi.getTransactions(PAGE_SIZE, (page - 1) * PAGE_SIZE, {
        status: status || undefined,
        type: type || undefined,
      }),
    [page, status, type],
    // The newest page follows the chain; older pages hold still while you read them.
    page === 1 ? 10_000 : undefined,
  );

  const setFilter = (key: "status" | "type", value: string) => {
    const updated = new URLSearchParams(params);
    updated.delete("page");
    if (value) updated.set(key, value);
    else updated.delete(key);
    setParams(updated);
  };

  const describe = [type ? spaced(type) : "", status].filter(Boolean).join(", ");

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Chain" title="Transactions">
        <p className="lede">Transfers, ride steps, mints and burns, newest first.</p>
      </PageHeader>
      <ErrorBanner message={error?.message} />
      <Panel
        flush
        actions={
          <div className="filters">
            <label className="select">
              <span className="visually-hidden">Type</span>
              <select value={type} onChange={(event) => setFilter("type", event.target.value)}>
                <option value="">All types</option>
                {TYPE_GROUPS.map((group) => (
                  <optgroup key={group.label} label={group.label}>
                    {group.types.map((t) => (
                      <option key={t} value={t}>
                        {spaced(t)}
                      </option>
                    ))}
                  </optgroup>
                ))}
              </select>
            </label>
          </div>
        }
      >
        {data ? (
          <TransactionsTable
            transactions={data.items}
            empty={describe ? `No ${describe} transactions.` : "No transactions indexed yet."}
          />
        ) : error ? null : (
          <Skeleton rows={12} />
        )}
        <Pagination page={page} hasMore={Boolean(data?.paging.has_more)} onChange={setPage} />
      </Panel>
    </div>
  );
}
