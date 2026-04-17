import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { Validator } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatNumber, shortHash } from "../utils/format";

const PAGE_SIZE = 20;

export function ValidatorsPage() {
  const [items, setItems] = useState<Validator[]>([]);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const response = await explorerApi.getValidators(PAGE_SIZE, offset);
        if (disposed) return;
        setItems(response.items);
        setHasMore(response.paging.has_more);
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
    <Panel title="Validators">
      <ErrorBanner message={error} />
      <table className="data-table">
        <thead>
          <tr>
            <th>Address</th>
            <th>Status</th>
            <th>Blocks Produced</th>
            <th>Peer ID</th>
          </tr>
        </thead>
        <tbody>
          {items.map((validator) => (
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
