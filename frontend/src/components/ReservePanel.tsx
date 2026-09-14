import type { Reserve } from "../api/types";
import { formatClt, formatRelativeTime } from "../utils/format";
import { Panel } from "./ui";

/**
 * The reserve position behind CLT.
 *
 * CLT claims to be fully reserved. Supply alone cannot support that claim — it is one side of the
 * equation, and a reader shown only "5,000,000 CLT exist" will treat it as proof of backing it does
 * not provide. So this panel always shows the pair: what was issued, and what is held against it.
 *
 * Every figure comes from a single reconciliation run, with that run's time. Live supply beside a
 * reconciled reserve would disagree routinely — a mint moves supply at once and the reserve figure
 * only at the next run — and a reader would read ordinary lag as a shortfall.
 */

/** What the three statuses mean to somebody who does not work here. */
function describe(status: string): { tone: string; headline: string; detail: string } {
  switch (status) {
    case "ok":
      return {
        tone: "ok",
        headline: "Fully reserved",
        detail: "Custody holds at least what the ledger owes, and every CLT on chain is accounted for.",
      };
    case "over_backed_drift":
      return {
        tone: "drift",
        headline: "Over-backed",
        detail:
          "Custody exceeds what the ledger owes. Ordinary for a few seconds around a mint or a burn, when one side has recorded something the other has not yet.",
      };
    case "mismatch":
      return {
        tone: "bad",
        headline: "Mismatch",
        detail:
          "Either CLT exists on chain that the ledger does not account for, or custody holds less than the ledger owes. Minting halts automatically on this.",
      };
    default:
      return { tone: "drift", headline: status, detail: "Unrecognised status." };
  }
}

export function ReservePanel({ reserve }: { reserve: Reserve | null }) {
  // No treasury behind this deployment: there is nothing to claim, so claim nothing.
  if (!reserve || !reserve.configured) return null;

  if (reserve.available === false) {
    return (
      <Panel title="Reserve">
        <p className="reserve-unavailable">
          The reserve position could not be read. Nothing is shown rather than the last known
          figures — a verification that is not happening should not look like one that is.
        </p>
      </Panel>
    );
  }

  const run = reserve.last_run;
  if (!run) {
    return (
      <Panel title="Reserve">
        <p className="reserve-unavailable">
          No reconciliation has run against this chain yet.
        </p>
      </Panel>
    );
  }

  const { tone, headline, detail } = describe(run.status);

  return (
    <Panel title="Reserve">
      <div className={`reserve-status reserve-status--${tone}`}>
        <strong>{headline}</strong>
        <span>{detail}</span>
      </div>

      <dl className="reserve-figures">
        <div>
          <dt>CLT issued</dt>
          <dd>{formatClt(run.treasury_minted)}</dd>
        </div>
        <div>
          <dt>Owed by the ledger</dt>
          <dd>{formatClt(run.ledger_liability)}</dd>
        </div>
        <div>
          <dt>Held in custody</dt>
          <dd>{formatClt(run.custody_reported)}</dd>
        </div>
      </dl>

      <p className="reserve-meta">
        As of {formatRelativeTime(run.run_at)}
        {typeof reserve.cache_age_seconds === "number" && reserve.cache_age_seconds > 0
          ? ` · read ${reserve.cache_age_seconds}s ago`
          : ""}
        {run.onchain_supply !== run.treasury_minted
          ? ` · ${formatClt(run.genesis_allocation)} allocated at genesis is excluded`
          : ""}
      </p>
    </Panel>
  );
}
