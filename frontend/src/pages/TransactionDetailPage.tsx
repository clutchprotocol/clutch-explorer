import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { TransactionDetail } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatRelativeTime, shortHash } from "../utils/format";

export function TransactionDetailPage() {
  const { hash = "" } = useParams();
  const [item, setItem] = useState<TransactionDetail | null>(null);
  const [latestHeight, setLatestHeight] = useState<number | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const [response, stats] = await Promise.all([
          explorerApi.getTransactionByHash(hash),
          explorerApi.getStats(),
        ]);
        if (disposed) return;
        setItem(response);
        setLatestHeight(stats.latest_height);
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
  }, [hash]);

  useEffect(() => {
    if (!item) return;

    let disposed = false;
    const poll = async () => {
      try {
        const stats = await explorerApi.getStats();
        if (!disposed) {
          setLatestHeight(stats.latest_height);
        }
      } catch {
        // Keep existing confirmation value on transient polling failures.
      }
    };

    const intervalId = window.setInterval(poll, 5000);
    return () => {
      disposed = true;
      window.clearInterval(intervalId);
    };
  }, [item]);

  if (loading) return <LoadingState />;
  if (!item) return <ErrorBanner message="Transaction not found" />;

  const confirmations =
    latestHeight === null
      ? null
      : Math.max(0, latestHeight - item.block_height + 1);
  const isConfirmed = item.status.toLowerCase() === "confirmed";
  const statusLabel =
    isConfirmed && confirmations !== null
      ? `Confirmed (${confirmations} confirmations)`
      : item.status;

  return (
    <Panel title={`Transaction ${shortHash(item.hash)}`}>
      <ErrorBanner message={error} />
      <dl className="detail-grid">
        <dt>Hash</dt>
        <dd>{item.hash}</dd>
        <dt>Status</dt>
        <dd>
          <span className={`status-pill ${item.status}`}>{statusLabel}</span>
          {item.is_ride_related && (
            <span className="ride-badge" title={item.function_call_type}>
              🚕 RIDE
            </span>
          )}
        </dd>
        <dt>Confirmations</dt>
        <dd>{isConfirmed ? (confirmations ?? "-") : 0}</dd>
        <dt>Action Type</dt>
        <dd>{item.function_call_type}</dd>
        <dt>Block</dt>
        <dd>
          <Link to={`/blocks/${item.block_height}`}>{item.block_height}</Link>
        </dd>
        <dt>From</dt>
        <dd>
          <Link to={`/address/${item.from}`}>{item.from}</Link>
        </dd>
        <dt>To</dt>
        <dd>
          <Link to={`/address/${item.to}`}>{item.to}</Link>
        </dd>
        <dt>Amount</dt>
        <dd>{item.amount}</dd>
        <dt>Fee</dt>
        <dd>{item.fee}</dd>
        <dt>Nonce</dt>
        <dd>{item.nonce}</dd>
        <dt>Tx Index</dt>
        <dd>{item.tx_index}</dd>
        <dt>Age</dt>
        <dd>{formatRelativeTime(item.timestamp)}</dd>
      </dl>
    </Panel>
  );
}
