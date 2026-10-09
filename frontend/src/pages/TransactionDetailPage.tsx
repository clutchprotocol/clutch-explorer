import { useParams } from "react-router-dom";
import { explorerApi } from "../api/client";
import {
  AddressLink,
  Amount,
  BlockLink,
  DetailList,
  DetailRow,
  ErrorBanner,
  FullValue,
  LoadingState,
  NotFoundState,
  PageHeader,
  Panel,
  StatusPill,
  TimeAgo,
  TypeBadge,
} from "../components/ui";
import { useApi } from "../hooks/useApi";
import { formatDateTime, formatHexAddress } from "../utils/format";

/** One line per transaction type, in the words a rider or driver would use. */
const TYPE_MEANING: Record<string, string> = {
  Transfer: "Sends CLT from one account to another.",
  WalletTransfer: "Sends CLT from one account to another, signed in MetaMask or another Ethereum wallet.",
  RideRequest: "A passenger asks for a ride and sets the fare aside.",
  RideOffer: "A driver offers to take a requested ride.",
  RideAcceptance: "The passenger accepts a driver's offer.",
  RidePay: "The passenger pays the driver for the ride. Referrer fees go to the apps involved.",
  RideCancel: "A ride in progress is cancelled.",
  RideRequestCancel: "The passenger withdraws a ride request.",
  Mint: "The treasury issues CLT against USDT received in custody.",
  Burn: "CLT is destroyed to redeem it for USDT.",
  ChainInit: "Genesis: sets the chain's consensus parameters.",
};

export function TransactionDetailPage() {
  const { hash = "" } = useParams();
  const tx = useApi(() => explorerApi.getTransactionByHash(hash), [hash]);
  // Polled separately so the confirmation count climbs while the page is open.
  const stats = useApi(() => explorerApi.getStats(), [], 5000);

  if (tx.loading) return <LoadingState />;
  if (tx.error?.isNotFound) return <NotFoundState what="transaction" id={hash} />;
  if (!tx.data) return <ErrorBanner message={tx.error?.message ?? "The transaction could not be loaded."} />;

  const item = tx.data;
  const latestHeight = stats.data?.latest_height;
  const isConfirmed = item.status.toLowerCase() === "confirmed";
  const confirmations =
    isConfirmed && latestHeight !== undefined ? Math.max(0, latestHeight - item.block_height + 1) : null;

  const referrer = formatHexAddress(item.referrer);
  const requestReferrer = formatHexAddress(item.request_referrer);
  const offerReferrer = formatHexAddress(item.offer_referrer);
  const referrerFees = (item.request_referrer_fee ?? 0) + (item.offer_referrer_fee ?? 0);

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Transaction" title={<TypeBadge type={item.function_call_type} isRide={item.is_ride_related} />}>
        <p className="lede">{TYPE_MEANING[item.function_call_type] ?? item.function_call_type}</p>
      </PageHeader>

      <section className="transfer-card" aria-label="Parties and amount">
        <div className="party">
          <span className="eyebrow">From</span>
          <AddressLink address={item.from} />
        </div>
        <div className="transfer-amount">
          {item.amount > 0 ? <Amount value={item.amount} /> : <span className="muted">No value moved</span>}
          <span className="transfer-arrow" aria-hidden="true" />
        </div>
        <div className="party">
          <span className="eyebrow">To</span>
          <AddressLink address={item.to} />
        </div>
      </section>

      <Panel title="Details">
        <DetailList>
          <DetailRow label="Hash">
            <FullValue value={item.hash} />
          </DetailRow>
          <DetailRow label="Status">
            <span className="inline-group">
              <StatusPill status={item.status} />
              {confirmations !== null ? (
                <span className="muted">{confirmations.toLocaleString()} confirmations</span>
              ) : null}
            </span>
          </DetailRow>
          <DetailRow label="Block">
            <BlockLink height={item.block_height} /> <span className="muted">· position {item.tx_index}</span>
          </DetailRow>
          <DetailRow label="Time">
            {formatDateTime(item.timestamp)} <span className="muted">(<TimeAgo value={item.timestamp} />)</span>
          </DetailRow>
          <DetailRow label="From">
            <AddressLink address={item.from} full />
          </DetailRow>
          <DetailRow label="To">
            <AddressLink address={item.to} full />
          </DetailRow>
          <DetailRow label="Amount">
            <Amount value={item.amount} />
          </DetailRow>
          <DetailRow label="Network fee">
            {item.fee > 0 ? (
              <>
                <Amount value={item.fee} /> <span className="muted">to the block producer</span>
              </>
            ) : (
              // The indexer writes 0 for every transaction today: the node's block payload carries
              // no per-transaction fee. Saying "$0.00" would claim the transaction was free.
              <span className="muted">Not recorded by the explorer yet</span>
            )}
          </DetailRow>
          <DetailRow label="Sender nonce">{item.nonce}</DetailRow>
          {referrer ? (
            <DetailRow label="Referrer app">
              <AddressLink address={referrer} full />
            </DetailRow>
          ) : null}
        </DetailList>
      </Panel>

      {item.function_call_type === "RidePay" && referrerFees > 0 ? (
        <Panel title="Referrer fees">
          <p className="muted small">
            Apps that brought the passenger and the driver to the network each earn a share of the fare.
          </p>
          <DetailList>
            {requestReferrer && item.request_referrer_fee ? (
              <DetailRow label="Passenger's app">
                <span className="inline-group">
                  <AddressLink address={requestReferrer} /> <Amount value={item.request_referrer_fee} />
                </span>
              </DetailRow>
            ) : null}
            {offerReferrer && item.offer_referrer_fee ? (
              <DetailRow label="Driver's app">
                <span className="inline-group">
                  <AddressLink address={offerReferrer} /> <Amount value={item.offer_referrer_fee} />
                </span>
              </DetailRow>
            ) : null}
            <DetailRow label="Total">
              <Amount value={referrerFees} />
            </DetailRow>
          </DetailList>
        </Panel>
      ) : null}
    </div>
  );
}
