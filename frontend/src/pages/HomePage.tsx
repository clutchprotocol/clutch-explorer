import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { BlockListItem, Stats, TransactionListItem, Validator } from "../api/types";
import { ErrorBanner, LoadingState, Panel, StatCard } from "../components/ui";
import { formatNumber, formatRelativeTime, shortHash } from "../utils/format";

export function HomePage() {
  const [stats, setStats] = useState<Stats | null>(null);
  const [blocks, setBlocks] = useState<BlockListItem[]>([]);
  const [txs, setTxs] = useState<TransactionListItem[]>([]);
  const [validators, setValidators] = useState<Validator[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const [statsRes, blocksRes, txRes, validatorsRes] = await Promise.all([
          explorerApi.getStats(),
          explorerApi.getBlocks(8, 0),
          explorerApi.getTransactions(8, 0),
          explorerApi.getValidators(6, 0),
        ]);
        if (disposed) return;
        setStats(statsRes);
        setBlocks(blocksRes.items);
        setTxs(txRes.items);
        setValidators(validatorsRes.items);
        setError("");
      } catch (err) {
        if (!disposed) setError((err as Error).message);
      } finally {
        if (!disposed) setLoading(false);
      }
    };
    load();
    const id = setInterval(load, 10000);
    return () => {
      disposed = true;
      clearInterval(id);
    };
  }, []);

  if (loading) return <LoadingState />;

  return (
    <div className="page-grid">
      <ErrorBanner message={error} />
      {stats ? (
        <section className="stats-grid">
          <StatCard label="Latest Block" value={formatNumber(stats.latest_height)} />
          <StatCard label="Total Transactions" value={formatNumber(stats.total_transactions)} />
          <StatCard label="TPS" value={stats.tx_per_second.toFixed(2)} />
          <StatCard label="Avg Block Time" value={`${stats.avg_block_time_seconds.toFixed(1)}s`} />
          <StatCard label="Active Validators" value={stats.active_validators} />
        </section>
      ) : null}

      <Panel title="Latest Blocks">
        <table className="data-table">
          <thead>
            <tr>
              <th>Height</th>
              <th>Hash</th>
              <th>Txs</th>
              <th>Producer</th>
              <th>Age</th>
            </tr>
          </thead>
          <tbody>
            {blocks.map((block) => (
              <tr key={block.hash}>
                <td>
                  <Link to={`/blocks/${block.height}`}>{block.height}</Link>
                </td>
                <td>{shortHash(block.hash)}</td>
                <td>{block.tx_count}</td>
                <td>
                  <Link to={`/address/${block.producer}`}>{shortHash(block.producer)}</Link>
                </td>
                <td>{formatRelativeTime(block.timestamp)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>

      <Panel title="Latest Transactions">
        <table className="data-table">
          <thead>
            <tr>
              <th>Hash</th>
              <th>Block</th>
              <th>From</th>
              <th>To</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {txs.map((tx) => (
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
                <td>
                  <span className={`status-pill ${tx.status}`}>{tx.status}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>

      <Panel title="Validators">
        <table className="data-table">
          <thead>
            <tr>
              <th>Address</th>
              <th>Status</th>
              <th>Blocks</th>
              <th>Peer</th>
            </tr>
          </thead>
          <tbody>
            {validators.map((validator) => (
              <tr key={validator.address}>
                <td>
                  <Link to={`/address/${validator.address}`}>{shortHash(validator.address)}</Link>
                </td>
                <td>{validator.is_active ? "active" : "inactive"}</td>
                <td>{formatNumber(validator.blocks_produced)}</td>
                <td>{validator.peer_id}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>
    </div>
  );
}
