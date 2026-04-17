import { ReactNode } from "react";

export function Panel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>{title}</h2>
      </div>
      {children}
    </section>
  );
}

export function StatCard({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="stat-card">
      <span className="stat-label">{label}</span>
      <strong className="stat-value">{value}</strong>
    </div>
  );
}

export function ErrorBanner({ message }: { message: string }) {
  if (!message) return null;
  return <p className="error-banner">{message}</p>;
}

export function LoadingState() {
  return <p className="muted">Loading...</p>;
}
