import { Link, useParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import { TransactionsTable } from "../components/tables";
import {
  AddressLink,
  Amount,
  DetailList,
  DetailRow,
  ErrorBanner,
  FullValue,
  LoadingState,
  NotFoundState,
  PageHeader,
  Panel,
  TimeAgo,
} from "../components/ui";
import { useApi } from "../hooks/useApi";
import { formatDateTime } from "../utils/format";

export function BlockDetailPage() {
  const { id = "" } = useParams();
  const { data, error, loading } = useApi(async () => {
    const [block, stats] = await Promise.all([
      explorerApi.getBlockById(id),
      explorerApi.getStats().catch(() => null),
    ]);
    // A block holds few transactions (one per sender), so one page of 100 is all of them.
    const txs = block.tx_count > 0 ? (await explorerApi.getTransactions(100, 0, { block: block.height })).items : [];
    return { block, txs, latestHeight: stats?.latest_height };
  }, [id]);

  if (loading) return <LoadingState />;
  if (error?.isNotFound) return <NotFoundState what="block" id={id} />;
  if (!data) return <ErrorBanner message={error?.message ?? "The block could not be loaded."} />;

  const { block, txs, latestHeight } = data;
  const isTip = latestHeight !== undefined && block.height >= latestHeight;

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Block" title={`#${block.height.toLocaleString()}`}>
        <div className="stepper">
          {block.height > 0 ? (
            <Link className="button button--ghost" to={`/blocks/${block.height - 1}`}>
              ← #{(block.height - 1).toLocaleString()}
            </Link>
          ) : (
            <span className="button button--ghost is-disabled">← Previous</span>
          )}
          {isTip ? (
            <span className="button button--ghost is-disabled" title="This is the newest indexed block">
              Newest block
            </span>
          ) : (
            <Link className="button button--ghost" to={`/blocks/${block.height + 1}`}>
              #{(block.height + 1).toLocaleString()} →
            </Link>
          )}
        </div>
      </PageHeader>

      <Panel title="Overview">
        <DetailList>
          <DetailRow label="Hash">
            <FullValue value={block.hash} />
          </DetailRow>
          <DetailRow label="Parent">
            {block.height === 0 ? (
              <span className="muted">None (genesis)</span>
            ) : (
              <FullValue value={block.parent_hash} to={`/blocks/${block.height - 1}`} />
            )}
          </DetailRow>
          <DetailRow label="Produced by">
            {block.height === 0 ? <span className="muted">Genesis</span> : <AddressLink address={block.producer} full />}
          </DetailRow>
          <DetailRow label="Time">
            {formatDateTime(block.timestamp)} <span className="muted">(<TimeAgo value={block.timestamp} />)</span>
          </DetailRow>
          <DetailRow label="Transactions">{block.tx_count}</DetailRow>
          <DetailRow label="Fees to producer">
            <Amount value={block.total_fees} />
          </DetailRow>
          {latestHeight !== undefined ? (
            <DetailRow label="Confirmations">{Math.max(0, latestHeight - block.height + 1).toLocaleString()}</DetailRow>
          ) : null}
        </DetailList>
      </Panel>

      <Panel title={`Transactions (${block.tx_count})`} flush>
        <TransactionsTable transactions={txs} showBlock={false} empty="This block carries no transactions." />
      </Panel>
    </div>
  );
}
