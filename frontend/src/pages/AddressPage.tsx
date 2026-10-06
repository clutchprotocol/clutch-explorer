import { Link, useParams, useSearchParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { AccountActivity } from "../api/types";
import { TransactionsTable } from "../components/tables";
import {
  AddressLink,
  Amount,
  BlockLink,
  CopyButton,
  EmptyState,
  ErrorBanner,
  LoadingState,
  NotFoundState,
  PageHeader,
  Pagination,
  Panel,
  Skeleton,
  StatCard,
  Table,
  TimeAgo,
  TxLink,
} from "../components/ui";
import { useApi } from "../hooks/useApi";
import { usePage } from "../hooks/usePage";
import { formatHexAddress, formatNumber } from "../utils/format";

const PAGE_SIZE = 20;

function ActivityTable({ rows }: { rows: AccountActivity[] }) {
  if (rows.length === 0) return <EmptyState>No balance changes indexed for this address yet.</EmptyState>;
  return (
    <Table>
      <thead>
        <tr>
          <th>What</th>
          <th className="num">Change</th>
          <th>Counterparty</th>
          <th>Transaction</th>
          <th>Block</th>
          <th>Age</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row, idx) => (
          <tr key={`${row.kind}-${row.block_height}-${row.tx_hash ?? "block"}-${idx}`}>
            <td>{row.label}</td>
            <td className="num">
              <Amount value={row.amount} sign={row.direction} />
            </td>
            <td>{row.counterparty ? <AddressLink address={row.counterparty} /> : <span className="muted">—</span>}</td>
            <td>{row.tx_hash ? <TxLink hash={row.tx_hash} /> : <span className="muted">—</span>}</td>
            <td>
              <BlockLink height={row.block_height} />
            </td>
            <td className="nowrap">
              <TimeAgo value={row.timestamp} />
            </td>
          </tr>
        ))}
      </tbody>
    </Table>
  );
}

export function AddressPage() {
  const { address: rawAddress = "" } = useParams();
  const address = formatHexAddress(rawAddress) ?? rawAddress;
  const [params] = useSearchParams();
  const tab = params.get("tab") === "transactions" ? "transactions" : "activity";
  const [page, setPage] = usePage();
  const offset = (page - 1) * PAGE_SIZE;

  const account = useApi(() => explorerApi.getAccountByAddress(address), [address]);
  const list = useApi(
    async () =>
      tab === "transactions"
        ? { kind: "transactions" as const, ...(await explorerApi.getTransactions(PAGE_SIZE, offset, { address })) }
        : { kind: "activity" as const, ...(await explorerApi.getAccountActivity(address, PAGE_SIZE, offset)) },
    [address, tab, offset],
  );

  if (account.loading) return <LoadingState />;
  if (account.error?.isNotFound) return <NotFoundState what="account" id={address} />;
  if (!account.data) return <ErrorBanner message={account.error?.message ?? "The account could not be loaded."} />;

  const info = account.data;
  const tabLink = (name: string) => {
    const next = new URLSearchParams();
    if (name !== "activity") next.set("tab", name);
    const query = next.toString();
    return query ? `?${query}` : "?";
  };

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Account" title={<span className="mono address-title">{address}</span>}>
        <CopyButton value={address} label="Copy address" />
      </PageHeader>

      <section className="stats-grid stats-grid--3" aria-label="Account summary">
        <StatCard label="Balance" value={<Amount value={info.balance} />} hint={`${formatNumber(info.balance)} CLT base units`} />
        <StatCard label="Transactions" value={formatNumber(info.tx_count)} hint={`nonce ${info.nonce}`} />
        <StatCard label="Balance changes" value={formatNumber(info.activity_count)} hint="fees, payments, mints" />
      </section>

      <Panel flush>
        <div className="tabs" role="tablist">
          <Link role="tab" aria-selected={tab === "activity"} className={tab === "activity" ? "is-active" : ""} to={tabLink("activity")}>
            Balance changes
          </Link>
          <Link
            role="tab"
            aria-selected={tab === "transactions"}
            className={tab === "transactions" ? "is-active" : ""}
            to={tabLink("transactions")}
          >
            Transactions
          </Link>
        </div>
        <ErrorBanner message={list.error?.message} />
        {!list.data ? (
          list.error ? null : <Skeleton rows={10} />
        ) : list.data.kind === "activity" ? (
          <ActivityTable rows={list.data.items} />
        ) : (
          <TransactionsTable transactions={list.data.items} perspective={address} empty="No transactions for this address yet." />
        )}
        <Pagination
          page={page}
          hasMore={Boolean(list.data?.paging.has_more)}
          onChange={setPage}
        />
      </Panel>
    </div>
  );
}
