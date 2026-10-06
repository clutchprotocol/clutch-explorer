import { explorerApi } from "../api/client";
import { AddressLink, CopyButton, ErrorBanner, PageHeader, Panel, Skeleton, Table } from "../components/ui";
import { useApi } from "../hooks/useApi";
import { formatNumber, shortHash } from "../utils/format";

export function ValidatorsPage() {
  // The authority set is small (three today), so one page holds all of it.
  const { data, error } = useApi(() => explorerApi.getValidators(100, 0), []);
  const validators = data?.items ?? [];
  const total = validators.reduce((sum, v) => sum + v.blocks_produced, 0) || 1;
  // The indexer knows producers by address only, so peer IDs are empty today; skip the column then.
  const hasPeerIds = validators.some((v) => v.peer_id);

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Consensus" title="Validators">
        <p className="lede">
          Clutch runs proof of authority (Aura): a fixed set of validators take turns, each producing
          the block for its time slot. An even share of blocks means every validator is keeping up.
        </p>
      </PageHeader>
      <ErrorBanner message={error?.message} />
      <Panel flush>
        {!data ? (
          error ? null : <Skeleton rows={4} />
        ) : (
          <Table>
            <thead>
              <tr>
                <th>Validator</th>
                <th>Status</th>
                <th className="num">Blocks produced</th>
                <th className="share-col">Share</th>
                {hasPeerIds ? <th>Peer ID</th> : null}
              </tr>
            </thead>
            <tbody>
              {validators.map((v) => {
                const share = v.blocks_produced / total;
                return (
                  <tr key={v.address}>
                    <td>
                      <AddressLink address={v.address} />
                    </td>
                    <td>
                      <span className={`status-pill status-pill--${v.is_active ? "confirmed" : "failed"}`}>
                        {v.is_active ? "active" : "inactive"}
                      </span>
                    </td>
                    <td className="num">{formatNumber(v.blocks_produced)}</td>
                    <td className="share-col">
                      <div className="share-cell">
                        <div className="shares-bar" aria-hidden="true">
                          <span style={{ width: `${share * 100}%` }} />
                        </div>
                        <span className="shares-pct">{(share * 100).toFixed(1)}%</span>
                      </div>
                    </td>
                    {hasPeerIds ? (
                      <td>
                        {v.peer_id ? (
                          <span className="inline-group">
                            <span className="mono muted" title={v.peer_id}>
                              {shortHash(v.peer_id, 10, 6)}
                            </span>
                            <CopyButton value={v.peer_id} label="Copy peer ID" />
                          </span>
                        ) : (
                          <span className="muted">—</span>
                        )}
                      </td>
                    ) : null}
                  </tr>
                );
              })}
            </tbody>
          </Table>
        )}
      </Panel>
    </div>
  );
}
