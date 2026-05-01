import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { BlockListItem } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatRelativeTime, shortHash } from "../utils/format";

const PAGE_SIZE = 20;

export function BlocksPage() {
  const [items, setItems] = useState<BlockListItem[]>([]);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const response = await explorerApi.getBlocks(PAGE_SIZE, offset);
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
  }, [offset]);

  if (loading) return <LoadingState />;

  return (
    <Panel title="Blocks">
      <ErrorBanner message={error} />
      <table className="data-table">
        <thead>
          <tr>
            <th>Height</th>
            <th>Hash</th>
            <th>Transactions</th>
            <th>Reward Recipient</th>
            <th>Block Reward</th>
            <th>Age</th>
          </tr>
        </thead>
        <tbody>
          {items.map((block) => (
            <tr key={block.hash}>
              <td>
                <Link to={`/blocks/${block.height}`}>{block.height}</Link>
              </td>
              <td>{shortHash(block.hash)}</td>
              <td>{block.tx_count}</td>
              <td>
                {block.height === 0 ? (
                  "Genesis"
                ) : (
                  <Link to={`/address/${block.reward_recipient}`}>
                    {shortHash(block.reward_recipient)}
                  </Link>
                )}
              </td>
              <td>{block.block_reward}</td>
              <td>{formatRelativeTime(block.timestamp)}</td>
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
