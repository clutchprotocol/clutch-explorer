import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { Account, AccountActivity, TransactionListItem } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatHexAddress, formatRelativeTime, shortHash } from "../utils/format";

export function AddressPage() {
  const { address = "" } = useParams();
  const [account, setAccount] = useState<Account | null>(null);
  const [transactions, setTransactions] = useState<TransactionListItem[]>([]);
  const [activity, setActivity] = useState<AccountActivity[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const [accountRes, txRes, activityRes] = await Promise.all([
          explorerApi.getAccountByAddress(address),
          explorerApi.getTransactions(20, 0, address),
          explorerApi.getAccountActivity(address, 20, 0),
        ]);
        if (disposed) return;
        setAccount(accountRes);
        setTransactions(txRes.items);
        setActivity(activityRes.items);
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
  }, [address]);

  if (loading) return <LoadingState />;
  if (!account) return <ErrorBanner message="Address not found" />;

  return (
    <div className="page-grid">
      <ErrorBanner message={error} />
      <Panel title="Address Overview">
        <dl className="detail-grid">
          <dt>Address</dt>
          <dd>{account.address}</dd>
          <dt>Balance</dt>
          <dd>{account.balance}</dd>
          <dt>Nonce</dt>
          <dd>{account.nonce}</dd>
          <dt>Transactions</dt>
          <dd>{account.tx_count}</dd>
          <dt>Activity</dt>
          <dd>{account.activity_count}</dd>
          <dt>Type</dt>
          <dd>{account.is_contract ? "Contract" : "EOA"}</dd>
        </dl>
      </Panel>

      <Panel title="Account Activity">
        <table className="data-table">
          <thead>
            <tr>
              <th>Type</th>
              <th>Amount</th>
              <th>Direction</th>
              <th>Counterparty</th>
              <th>Transaction</th>
              <th>Block</th>
              <th>Age</th>
            </tr>
          </thead>
          <tbody>
            {activity.length === 0 ? (
              <tr>
                <td colSpan={7}>No balance activity indexed for this address yet.</td>
              </tr>
            ) : (
              activity.map((row, idx) => (
                <tr key={`${row.kind}-${row.block_height}-${row.tx_hash ?? "block"}-${idx}`}>
                  <td>
                    <span className="badge">{row.label}</span>
                  </td>
                  <td>{row.direction === "in" ? "+" : "-"}
                    {row.amount}</td>
                  <td>{row.direction}</td>
                  <td>
                    {row.counterparty ? (
                      (() => {
                        const cp = formatHexAddress(row.counterparty) ?? row.counterparty;
                        return (
                          <Link to={`/address/${cp}`}>{shortHash(cp)}</Link>
                        );
                      })()
                    ) : (
                      "—"
                    )}
                  </td>
                  <td>
                    {row.tx_hash ? (
                      <Link to={`/txs/${row.tx_hash}`}>{shortHash(row.tx_hash)}</Link>
                    ) : (
                      "—"
                    )}
                  </td>
                  <td>
                    <Link to={`/blocks/${row.block_height}`}>{row.block_height}</Link>
                  </td>
                  <td>{formatRelativeTime(row.timestamp)}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </Panel>

      <Panel title="Address Transactions">
        <table className="data-table">
          <thead>
            <tr>
              <th>Hash</th>
              <th>Block</th>
              <th>From</th>
              <th>To</th>
              <th>Status</th>
              <th>Age</th>
            </tr>
          </thead>
          <tbody>
            {transactions.map((tx) => (
              <tr key={tx.hash}>
                <td>
                  <Link to={`/txs/${tx.hash}`}>{shortHash(tx.hash)}</Link>
                </td>
                <td>
                  <Link to={`/blocks/${tx.block_height}`}>{tx.block_height}</Link>
                </td>
                <td>{shortHash(tx.from)}</td>
                <td>{shortHash(tx.to)}</td>
                <td>{tx.status}</td>
                <td>{formatRelativeTime(tx.timestamp)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>
    </div>
  );
}
