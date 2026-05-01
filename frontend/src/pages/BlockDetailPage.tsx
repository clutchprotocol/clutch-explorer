import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { BlockDetail, TransactionListItem } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatRelativeTime, shortHash } from "../utils/format";

export function BlockDetailPage() {
  const { id = "" } = useParams();
  const navigate = useNavigate();
  const [block, setBlock] = useState<BlockDetail | null>(null);
  const [transactions, setTransactions] = useState<TransactionListItem[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      if (!disposed) {
        setLoading(true);
        setError("");
      }
      try {
        const [blockRes, txRes] = await Promise.all([
          explorerApi.getBlockById(id),
          explorerApi.getTransactions(10, 0),
        ]);
        if (disposed) return;
        setBlock(blockRes);
        setTransactions(txRes.items.filter((tx) => tx.block_height === blockRes.height));
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
  }, [id]);

  if (loading) return <LoadingState />;
  if (!block) return <ErrorBanner message="Block not found" />;

  return (
    <div className="page-grid">
      <ErrorBanner message={error} />
      <Panel title={`Block #${block.height}`}>
        <div className="pagination">
          <button
            onClick={() => navigate(`/blocks/${block.height - 1}`)}
            disabled={block.height === 0}
          >
            Previous Block
          </button>
          <button onClick={() => navigate(`/blocks/${block.height + 1}`)}>Next Block</button>
        </div>
        <dl className="detail-grid">
          <dt>Hash</dt>
          <dd>{block.hash}</dd>
          <dt>Parent</dt>
          <dd>
            <Link to={`/blocks/${block.parent_hash}`}>{shortHash(block.parent_hash)}</Link>
          </dd>
          <dt>Producer</dt>
          <dd>
            {block.height === 0 ? (
              "Genesis"
            ) : (
              <Link to={`/address/${block.producer}`}>{block.producer}</Link>
            )}
          </dd>
          <dt>Reward Recipient</dt>
          <dd>
            <Link to={`/address/${block.reward_recipient}`}>{block.reward_recipient}</Link>
          </dd>
          <dt>Block Reward</dt>
          <dd>{block.block_reward}</dd>
          <dt>Transactions</dt>
          <dd>{block.tx_count}</dd>
          <dt>Total Fees</dt>
          <dd>{block.total_fees}</dd>
          <dt>Age</dt>
          <dd>{formatRelativeTime(block.timestamp)}</dd>
        </dl>
      </Panel>
      <Panel title="Transactions in Block">
        <table className="data-table">
          <thead>
            <tr>
              <th>Hash</th>
              <th>From</th>
              <th>To</th>
              <th>Amount</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {transactions.map((tx) => (
              <tr key={tx.hash}>
                <td>
                  <Link to={`/txs/${tx.hash}`}>{shortHash(tx.hash)}</Link>
                </td>
                <td>
                  <Link to={`/address/${tx.from}`}>{shortHash(tx.from)}</Link>
                </td>
                <td>
                  <Link to={`/address/${tx.to}`}>{shortHash(tx.to)}</Link>
                </td>
                <td>{tx.amount}</td>
                <td>{tx.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>
    </div>
  );
}
