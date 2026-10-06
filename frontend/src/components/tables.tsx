import type { BlockListItem, TransactionListItem } from "../api/types";
import { formatHexAddress } from "../utils/format";
import {
  AddressLink,
  Amount,
  BlockLink,
  EmptyState,
  StatusPill,
  Table,
  TimeAgo,
  TxLink,
  TypeBadge,
} from "./ui";

export function BlocksTable({ blocks }: { blocks: BlockListItem[] }) {
  if (blocks.length === 0) return <EmptyState>No blocks indexed yet.</EmptyState>;
  return (
    <Table>
      <thead>
        <tr>
          <th>Block</th>
          <th>Hash</th>
          <th className="num">Txs</th>
          <th>Producer</th>
          <th>Age</th>
        </tr>
      </thead>
      <tbody>
        {blocks.map((block) => (
          <tr key={block.hash}>
            <td>
              <BlockLink height={block.height} />
            </td>
            <td className="mono muted" title={block.hash}>
              {block.hash.slice(0, 12)}…
            </td>
            <td className="num">
              {block.tx_count > 0 ? block.tx_count : <span className="muted">0</span>}
            </td>
            <td>
              {block.height === 0 ? <span className="muted">Genesis</span> : <AddressLink address={block.producer} />}
            </td>
            <td className="nowrap">
              <TimeAgo value={block.timestamp} />
            </td>
          </tr>
        ))}
      </tbody>
    </Table>
  );
}

/** `perspective` marks the rows as in or out for the address page. */
export function TransactionsTable({
  transactions,
  perspective,
  showBlock = true,
  empty = "No transactions indexed yet.",
}: {
  transactions: TransactionListItem[];
  perspective?: string;
  showBlock?: boolean;
  empty?: string;
}) {
  if (transactions.length === 0) return <EmptyState>{empty}</EmptyState>;
  const self = formatHexAddress(perspective);
  return (
    <Table>
      <thead>
        <tr>
          <th>Transaction</th>
          <th>Type</th>
          {showBlock ? <th>Block</th> : null}
          <th>From</th>
          <th aria-label="Direction" />
          <th>To</th>
          <th className="num">Amount</th>
          <th>Status</th>
          <th>Age</th>
        </tr>
      </thead>
      <tbody>
        {transactions.map((tx) => {
          const from = formatHexAddress(tx.from);
          const to = formatHexAddress(tx.to);
          const direction = !self ? undefined : from === self && to !== self ? "out" : to === self && from !== self ? "in" : undefined;
          return (
            <tr key={tx.hash}>
              <td>
                <TxLink hash={tx.hash} />
              </td>
              <td>
                <TypeBadge type={tx.function_call_type} isRide={tx.is_ride_related} />
              </td>
              {showBlock ? (
                <td>
                  <BlockLink height={tx.block_height} />
                </td>
              ) : null}
              <td>
                {from === self ? <span className="self-tag">this address</span> : <AddressLink address={tx.from} />}
              </td>
              <td className="arrow" aria-hidden="true">
                {direction === "in" ? <span className="dir dir--in">IN</span> : direction === "out" ? <span className="dir dir--out">OUT</span> : "→"}
              </td>
              <td>
                {to === self ? <span className="self-tag">this address</span> : <AddressLink address={tx.to} />}
              </td>
              <td className="num">
                {tx.amount > 0 ? <Amount value={tx.amount} /> : <span className="muted">—</span>}
              </td>
              <td>
                <StatusPill status={tx.status} />
              </td>
              <td className="nowrap">
                <TimeAgo value={tx.timestamp} />
              </td>
            </tr>
          );
        })}
      </tbody>
    </Table>
  );
}
