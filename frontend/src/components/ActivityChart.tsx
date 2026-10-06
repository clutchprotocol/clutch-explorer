import { useNavigate } from "react-router-dom";
import type { BlockListItem } from "../api/types";
import { formatRelativeTime } from "../utils/format";

/**
 * Transactions per block for the newest blocks, oldest on the left. Each bar opens its block.
 * An empty block still gets a stub, so a quiet chain reads as "producing, but idle" rather than
 * as missing data.
 */
export function ActivityChart({ blocks }: { blocks: BlockListItem[] }) {
  const navigate = useNavigate();
  if (blocks.length === 0) return null;

  const ordered = [...blocks].sort((a, b) => a.height - b.height);
  const max = Math.max(1, ...ordered.map((b) => b.tx_count));
  const total = ordered.reduce((sum, b) => sum + b.tx_count, 0);
  const busy = ordered.filter((b) => b.tx_count > 0).length;
  const width = 600;
  const height = 120;
  const gap = 2;
  const barWidth = (width - gap * (ordered.length - 1)) / ordered.length;

  return (
    <figure className="activity">
      <figcaption className="activity-caption">
        <span>
          <strong>{total}</strong> transactions in the last {ordered.length} blocks
        </span>
        <span className="muted small">
          {busy} of {ordered.length} blocks carried any
        </span>
      </figcaption>
      <svg
        className="activity-chart"
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
        role="img"
        aria-label={`Transactions per block for blocks ${ordered[0].height} to ${ordered[ordered.length - 1].height}`}
      >
        {ordered.map((block, i) => {
          const h = block.tx_count === 0 ? 3 : Math.max(8, (block.tx_count / max) * (height - 4));
          return (
            <rect
              key={block.hash}
              className={block.tx_count === 0 ? "activity-bar is-empty" : "activity-bar"}
              x={i * (barWidth + gap)}
              y={height - h}
              width={barWidth}
              height={h}
              rx={1.5}
              onClick={() => navigate(`/blocks/${block.height}`)}
            >
              <title>
                {`Block ${block.height}: ${block.tx_count} ${block.tx_count === 1 ? "tx" : "txs"}, ${formatRelativeTime(block.timestamp)}`}
              </title>
            </rect>
          );
        })}
      </svg>
      <div className="activity-axis small muted" aria-hidden="true">
        <span>#{ordered[0].height.toLocaleString()}</span>
        <span>#{ordered[ordered.length - 1].height.toLocaleString()}</span>
      </div>
    </figure>
  );
}
