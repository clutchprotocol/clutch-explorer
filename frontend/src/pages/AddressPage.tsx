import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import type { Account, TransactionListItem } from "../api/types";
import { ErrorBanner, LoadingState, Panel } from "../components/ui";
import { formatRelativeTime, shortHash } from "../utils/format";

export function AddressPage() {
  const { address = "" } = useParams();
  const [account, setAccount] = useState<Account | null>(null);
  const [transactions, setTransactions] = useState<TransactionListItem[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let disposed = false;
    const load = async () => {
      try {
        const [accountRes, txRes] = await Promise.all([
          explorerApi.getAccountByAddress(address),
          explorerApi.getTransactions(20, 0, address),
        ]);
        if (disposed) return;
        setAccount(accountRes);
        setTransactions(txRes.items);
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
          <dt>Type</dt>
          <dd>{account.is_contract ? "Contract" : "EOA"}</dd>
        </dl>
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
