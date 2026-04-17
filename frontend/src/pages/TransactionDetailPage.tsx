import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { TransactionDetail } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatRelativeTime, shortHash } from "../utils/format";

export function TransactionDetailPage() {
  const { hash = "" } = useParams();
  const [item, setItem] = useState<TransactionDetail | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const response = await explorerApi.getTransactionByHash(hash);
        if (disposed) return;
        setItem(response);
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

  if (loading) return <LoadingState />;
  if (!item) return <ErrorBanner message="Transaction not found" />;

  return (
    <Panel title={`Transaction ${shortHash(item.hash)}`}>
      <ErrorBanner message={error} />
      <dl className="detail-grid">
        <dt>Hash</dt>
        <dd>{item.hash}</dd>
        <dt>Status</dt>
        <dd>
          <span className={`status-pill ${item.status}`}>{item.status}</span>
        </dd>
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
