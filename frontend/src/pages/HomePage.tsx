import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { explorerApi } from "../api/client";
import { ActivityChart } from "../components/ActivityChart";
import { ReservePanel } from "../components/ReservePanel";
import { SearchBox } from "../components/SearchBox";
import {
  AddressLink,
  Amount,
  BlockLink,
  EmptyState,
  ErrorBanner,
  Panel,
  Skeleton,
  StatCard,
  TimeAgo,
  TxLink,
  TypeBadge,
} from "../components/ui";
import { useApi } from "../hooks/useApi";
import { formatNumber, networkLabel } from "../utils/format";

const REFRESH_MS = 10_000;

function LiveIndicator({ updatedAt, failing }: { updatedAt?: number; failing: boolean }) {
  const [, tick] = useState(0);
  useEffect(() => {
    const id = window.setInterval(() => tick((n) => n + 1), 1000);
    return () => window.clearInterval(id);
  }, []);
  if (!updatedAt) return null;
  const age = Math.max(0, Math.round((Date.now() - updatedAt) / 1000));
  return (
    <span className={`live${failing ? " live--stale" : ""}`}>
      <span className="live-dot" aria-hidden="true" />
      {failing ? `Not updating · last ${age}s ago` : `Live · updated ${age}s ago`}
    </span>
  );
}

export function HomePage() {
  const network = networkLabel();
  const { data, error, updatedAt } = useApi(
    async () => {
      const [stats, blocks, txs, validators, reserve] = await Promise.all([
        explorerApi.getStats(),
        explorerApi.getBlocks(60, 0),
        explorerApi.getTransactions(8, 0),
        explorerApi.getValidators(10, 0),
        // Settled separately: the reserve comes from another service, and its being down must
        // not blank the block list. The panel renders its own unavailable state instead.
        explorerApi.getReserve().catch(() => null),
      ]);
      return { stats, recent: blocks.items, blocks: blocks.items.slice(0, 8), txs: txs.items, validators: validators.items, reserve };
    },
    [],
    REFRESH_MS,
  );

  const stats = data?.stats;

  return (
    <div className="page-grid">
      <section className="hero">
        <div className="hero-copy">
          <span className="eyebrow">Clutch Protocol · {network.label}</span>
          <h1>Every ride, payment and block on the Clutch chain.</h1>
          <p>Look up a block, a transaction or an account, or watch the chain as it grows.</p>
        </div>
        <SearchBox size="large" />
      </section>

      <div className="section-head">
        <h2 className="visually-hidden">Network</h2>
        <LiveIndicator updatedAt={updatedAt} failing={Boolean(error && data)} />
      </div>
      <ErrorBanner message={error?.message} />

      <section className="stats-grid" aria-label="Network stats">
        <StatCard label="Latest block" value={stats ? formatNumber(stats.latest_height) : "—"} />
        <StatCard label="Transactions" value={stats ? formatNumber(stats.total_transactions) : "—"} hint="all time" />
        <StatCard
          label="Block time"
          value={stats && stats.avg_block_time_seconds > 0 ? `${stats.avg_block_time_seconds.toFixed(1)}s` : "—"}
          hint="last 100 blocks"
        />
        <StatCard
          label="Throughput"
          value={stats ? `${stats.tx_per_second.toFixed(2)}` : "—"}
          hint="tx per second"
        />
        <StatCard label="Validators" value={stats ? stats.active_validators : "—"} hint="active, proof of authority" />
      </section>

      {data && data.recent.length > 0 ? (
        <Panel title="Chain activity">
          <ActivityChart blocks={data.recent} />
        </Panel>
      ) : null}

      <ReservePanel reserve={data?.reserve} />

      <div className="two-col">
        <Panel
          title="Latest blocks"
          actions={
            <Link className="panel-link" to="/blocks">
              All blocks →
            </Link>
          }
        >
          {!data ? (
            <Skeleton rows={8} />
          ) : data.blocks.length === 0 ? (
            <EmptyState>No blocks indexed yet.</EmptyState>
          ) : (
            <ul className="feed">
              {data.blocks.map((block) => (
                <li key={block.hash}>
                  <span className="feed-tile feed-tile--block" aria-hidden="true">
                    Bk
                  </span>
                  <div className="feed-main">
                    <BlockLink height={block.height} />
                    <span className="muted small">
                      <TimeAgo value={block.timestamp} />
                    </span>
                  </div>
                  <div className="feed-side">
                    <span className="small">
                      {block.height === 0 ? "Genesis" : <>by <AddressLink address={block.producer} /></>}
                    </span>
                    <span className="small muted">
                      {block.tx_count} {block.tx_count === 1 ? "tx" : "txs"}
                    </span>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Panel>

        <Panel
          title="Latest transactions"
          actions={
            <Link className="panel-link" to="/txs">
              All transactions →
            </Link>
          }
        >
          {!data ? (
            <Skeleton rows={8} />
          ) : data.txs.length === 0 ? (
            <EmptyState>No transactions indexed yet.</EmptyState>
          ) : (
            <ul className="feed feed--tx">
              {data.txs.map((tx) => (
                <li key={tx.hash}>
                  <span className="feed-tile feed-tile--tx" aria-hidden="true">
                    Tx
                  </span>
                  <div className="feed-main">
                    <TxLink hash={tx.hash} />
                    <TypeBadge type={tx.function_call_type} isRide={tx.is_ride_related} />
                  </div>
                  <div className="feed-side">
                    <span className="small">
                      <AddressLink address={tx.from} /> → <AddressLink address={tx.to} />
                    </span>
                    <span className="small">
                      {tx.amount > 0 ? <Amount value={tx.amount} /> : <span className="muted">no value</span>}
                      <span className="muted"> · <TimeAgo value={tx.timestamp} /></span>
                    </span>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Panel>
      </div>

      {data && data.validators.length > 0 ? (
        <Panel
          title="Validators"
          actions={
            <Link className="panel-link" to="/validators">
              Details →
            </Link>
          }
        >
          <ValidatorShares validators={data.validators} />
        </Panel>
      ) : null}
    </div>
  );
}

export function ValidatorShares({
  validators,
}: {
  validators: { address: string; blocks_produced: number; is_active: boolean }[];
}) {
  const total = validators.reduce((sum, v) => sum + v.blocks_produced, 0) || 1;
  return (
    <ul className="shares">
      {validators.map((v) => {
        const share = v.blocks_produced / total;
        return (
          <li key={v.address}>
            <div className="shares-row">
              <span className={`dot ${v.is_active ? "dot--on" : "dot--off"}`} title={v.is_active ? "Active" : "Inactive"} />
              <AddressLink address={v.address} />
              <span className="muted small">{formatNumber(v.blocks_produced)} blocks</span>
              <span className="shares-pct">{(share * 100).toFixed(1)}%</span>
            </div>
            <div className="shares-bar" aria-hidden="true">
              <span style={{ width: `${share * 100}%` }} />
            </div>
          </li>
        );
      })}
    </ul>
  );
}
