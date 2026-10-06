import { ReactNode, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  formatCltPrecise,
  formatDateTime,
  formatHexAddress,
  formatRelativeTime,
  shortHash,
} from "../utils/format";

export function Panel({
  title,
  actions,
  children,
  flush = false,
}: {
  title?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  /** Let a table run to the panel's edges. */
  flush?: boolean;
}) {
  return (
    <section className={`panel${flush ? " panel--flush" : ""}`}>
      {title || actions ? (
        <div className="panel-header">
          {title ? <h2>{title}</h2> : <span />}
          {actions}
        </div>
      ) : null}
      {children}
    </section>
  );
}

export function PageHeader({
  eyebrow,
  title,
  children,
}: {
  eyebrow?: ReactNode;
  title: ReactNode;
  children?: ReactNode;
}) {
  return (
    <header className="page-header">
      {eyebrow ? <span className="eyebrow">{eyebrow}</span> : null}
      <h1>{title}</h1>
      {children}
    </header>
  );
}

export function StatCard({ label, value, hint }: { label: string; value: ReactNode; hint?: ReactNode }) {
  return (
    <div className="stat-card">
      <span className="stat-label">{label}</span>
      <strong className="stat-value">{value}</strong>
      {hint ? <span className="stat-hint">{hint}</span> : null}
    </div>
  );
}

export function ErrorBanner({ message }: { message?: string }) {
  if (!message) return null;
  return (
    <p className="error-banner" role="alert">
      {message}
    </p>
  );
}

export function EmptyState({ children }: { children: ReactNode }) {
  return <p className="empty-state">{children}</p>;
}

/** Grey bars in the shape of the content that is coming, so the layout does not jump. */
export function Skeleton({ rows = 6 }: { rows?: number }) {
  return (
    <div className="skeleton" aria-busy="true" aria-label="Loading">
      {Array.from({ length: rows }, (_, i) => (
        <span key={i} className="skeleton-bar" style={{ width: `${70 + ((i * 37) % 30)}%` }} />
      ))}
    </div>
  );
}

export function LoadingState() {
  return (
    <div className="page-grid">
      <div className="skeleton-title" />
      <Panel>
        <Skeleton />
      </Panel>
    </div>
  );
}

export function NotFoundState({ what, id }: { what: string; id?: string }) {
  return (
    <div className="not-found">
      <span className="eyebrow">Not found</span>
      <h1>No {what} here</h1>
      <p>
        {id ? (
          <>
            Nothing matching <code className="mono">{shortHash(id, 12, 8)}</code> has been indexed.
          </>
        ) : (
          "That page does not exist."
        )}{" "}
        It may be on another network, or not indexed yet.
      </p>
      <Link className="button" to="/">
        Back to the explorer
      </Link>
    </div>
  );
}

export function CopyButton({ value, label = "Copy" }: { value: string; label?: string }) {
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) return;
    const id = window.setTimeout(() => setCopied(false), 1500);
    return () => window.clearTimeout(id);
  }, [copied]);

  return (
    <button
      type="button"
      className={`copy-button${copied ? " is-copied" : ""}`}
      aria-label={copied ? "Copied" : `${label} ${value}`}
      title={copied ? "Copied" : label}
      onClick={() => {
        navigator.clipboard?.writeText(value).then(() => setCopied(true), () => undefined);
      }}
    >
      {copied ? (
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M3 8.5l3 3 7-7" fill="none" stroke="currentColor" strokeWidth="1.8" />
        </svg>
      ) : (
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <rect x="5" y="5" width="8.5" height="8.5" rx="1.5" fill="none" stroke="currentColor" strokeWidth="1.4" />
          <path d="M3 10.5V3.5A1 1 0 0 1 4 2.5h6.5" fill="none" stroke="currentColor" strokeWidth="1.4" />
        </svg>
      )}
    </button>
  );
}

/** A full value that wraps, with a copy button. For detail pages. */
export function FullValue({ value, to }: { value: string; to?: string }) {
  return (
    <span className="full-value">
      {to ? (
        <Link className="mono" to={to}>
          {value}
        </Link>
      ) : (
        <span className="mono">{value}</span>
      )}
      <CopyButton value={value} />
    </span>
  );
}

export function AddressLink({
  address,
  full = false,
  label,
}: {
  address: string | null | undefined;
  full?: boolean;
  label?: string;
}) {
  const normalized = formatHexAddress(address);
  if (!normalized || normalized === "0xunknown") return <span className="muted">—</span>;
  if (full) return <FullValue value={normalized} to={`/address/${normalized}`} />;
  return (
    <Link className="mono" to={`/address/${normalized}`} title={normalized}>
      {label ?? shortHash(normalized)}
    </Link>
  );
}

export function TxLink({ hash }: { hash: string }) {
  return (
    <Link className="mono" to={`/txs/${hash}`} title={hash}>
      {shortHash(hash)}
    </Link>
  );
}

export function BlockLink({ height }: { height: number }) {
  return (
    <Link className="mono" to={`/blocks/${height}`}>
      {height.toLocaleString()}
    </Link>
  );
}

export function TimeAgo({ value }: { value: string }) {
  return (
    <time dateTime={value} title={formatDateTime(value)}>
      {formatRelativeTime(value)}
    </time>
  );
}

export function Amount({ value, sign }: { value: number; sign?: "in" | "out" }) {
  const prefix = sign === "in" ? "+" : sign === "out" ? "−" : "";
  return (
    <span
      className={`amount${sign ? ` amount--${sign}` : ""}`}
      title={`${value.toLocaleString()} CLT base units`}
    >
      {prefix}
      {formatCltPrecise(value)}
    </span>
  );
}

const TYPE_TONES: Record<string, string> = {
  Transfer: "neutral",
  Mint: "mint",
  Burn: "burn",
  ChainInit: "neutral",
};

/** What a transaction does, readable at a glance; ride steps share one colour. */
export function TypeBadge({ type, isRide }: { type: string; isRide?: boolean }) {
  const tone = isRide || type.startsWith("Ride") ? "ride" : TYPE_TONES[type] ?? "neutral";
  const label = type.replace(/([a-z])([A-Z])/g, "$1 $2");
  return <span className={`type-badge type-badge--${tone}`}>{label}</span>;
}

export function StatusPill({ status }: { status: string }) {
  const tone = status.toLowerCase();
  return <span className={`status-pill status-pill--${tone}`}>{tone}</span>;
}

export function Pagination({
  page,
  hasMore,
  onChange,
}: {
  page: number;
  hasMore: boolean;
  onChange: (page: number) => void;
}) {
  if (page === 1 && !hasMore) return null;
  return (
    <nav className="pagination" aria-label="Pagination">
      <button type="button" onClick={() => onChange(1)} disabled={page === 1}>
        Newest
      </button>
      <button type="button" onClick={() => onChange(page - 1)} disabled={page === 1}>
        ← Newer
      </button>
      <span className="pagination-page">Page {page}</span>
      <button type="button" onClick={() => onChange(page + 1)} disabled={!hasMore}>
        Older →
      </button>
    </nav>
  );
}

export function DetailList({ children }: { children: ReactNode }) {
  return <dl className="detail-list">{children}</dl>;
}

export function DetailRow({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <div className="detail-row">
      <dt>{label}</dt>
      <dd>{children}</dd>
    </div>
  );
}

export function Table({ children }: { children: ReactNode }) {
  return (
    <div className="table-wrap">
      <table className="data-table">{children}</table>
    </div>
  );
}
