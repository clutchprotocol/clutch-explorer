import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { TransactionListItem } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatRelativeTime, shortHash } from "../utils/format";

const PAGE_SIZE = 20;

export function TransactionsPage() {
  const [items, setItems] = useState<TransactionListItem[]>([]);
  const [offset, setOffset] = useState(0);
  const [status, setStatus] = useState("");
  const [address, setAddress] = useState("");
  const [hasMore, setHasMore] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const response = await explorerApi.getTransactions(
          PAGE_SIZE,
          offset,
          address || undefined,
          status || undefined
        );
        if (disposed) return;
        setItems(response.items);
        setHasMore(response.paging.has_more);
        setError("");
      } catch (err) {
        if (!disposed) setError((err as Error).message);
      } finally {
        if (!disposed) setLoading(false);
      }
    };
    load();
    return () => {
      disposed = true;
    };
  }, [offset, status, address]);

  if (loading) return <LoadingState />;

  return (
    <Panel title="Transactions">
      <ErrorBanner message={error} />
      <div className="toolbar">
        <input
          placeholder="Filter by address"
          value={address}
          onChange={(event) => setAddress(event.target.value)}
        />
        <select value={status} onChange={(event) => setStatus(event.target.value)}>
          <option value="">Any status</option>
          <option value="confirmed">confirmed</option>
          <option value="pending">pending</option>
          <option value="failed">failed</option>
        </select>
      </div>
      <table className="data-table">
        <thead>
          <tr>
            <th>Hash</th>
            <th>Block</th>
            <th>From</th>
            <th>To</th>
            <th>Amount</th>
            <th>Status</th>
            <th>Age</th>
          </tr>
        </thead>
        <tbody>
          {items.map((tx) => (
            <tr key={tx.hash}>
              <td>
                <Link to={`/txs/${tx.hash}`}>{shortHash(tx.hash)}</Link>
              </td>
              <td>
                <Link to={`/blocks/${tx.block_height}`}>{tx.block_height}</Link>
              </td>
              <td>
                <Link to={`/address/${tx.from}`}>{shortHash(tx.from)}</Link>
              </td>
              <td>
                <Link to={`/address/${tx.to}`}>{shortHash(tx.to)}</Link>
              </td>
              <td>{tx.amount}</td>
              <td>
                <span className={`status-pill ${tx.status}`}>{tx.status}</span>
              </td>
              <td>{formatRelativeTime(tx.timestamp)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="pagination">
        <button onClick={() => setOffset((current) => Math.max(0, current - PAGE_SIZE))}>
          Previous
        </button>
        <span>Offset {offset}</span>
        <button onClick={() => setOffset((current) => current + PAGE_SIZE)} disabled={!hasMore}>
          Next
        </button>
      </div>
    </Panel>
  );
}
