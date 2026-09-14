export type Paging = {
  limit: number;
  offset: number;
  total: number;
  has_more: boolean;
};

export type ListResponse<T> = {
  items: T[];
  paging: Paging;
};

export type BlockListItem = {
  height: number;
  hash: string;
  tx_count: number;
  producer: string;
  reward_recipient: string;
  block_reward: number;
  timestamp: string;
};

export type BlockDetail = BlockListItem & {
  parent_hash: string;
  total_fees: number;
};

export type TransactionListItem = {
  hash: string;
  block_height: number;
  from: string;
  to: string;
  amount: number;
  fee: number;
  status: string;
  function_call_type: string;
  is_ride_related: boolean;
  timestamp: string;
  referrer?: string | null;
  request_referrer_fee?: number;
  offer_referrer_fee?: number;
};

export type TransactionDetail = TransactionListItem & {
  nonce: number;
  tx_index: number;
  request_referrer?: string | null;
  offer_referrer?: string | null;
};

export type Account = {
  address: string;
  balance: number;
  nonce: number;
  tx_count: number;
  activity_count: number;
  is_contract: boolean;
};

export type AccountActivity = {
  address: string;
  kind: string;
  label: string;
  delta: number;
  direction: "in" | "out";
  amount: number;
  tx_hash?: string | null;
  block_height: number;
  tx_index?: number | null;
  function_call_type?: string | null;
  counterparty?: string | null;
  timestamp: string;
};

export type Validator = {
  address: string;
  is_active: boolean;
  blocks_produced: number;
  peer_id: string;
};

export type Stats = {
  latest_height: number;
  tx_per_second: number;
  total_transactions: number;
  active_validators: number;
  avg_block_time_seconds: number;
};

export type SearchResult = {
  kind: "block" | "transaction" | "account";
  identifier: string;
  summary: string;
};

/// The reserve position behind CLT, from the treasury's last reconciliation run.
///
/// Every number comes from that single run, so they are consistent with each other — the reason
/// live chain supply is deliberately not mixed in here.
export type ReserveRun = {
  run_at: string;
  onchain_supply: number;
  genesis_allocation: number;
  treasury_minted: number;
  ledger_liability: number;
  custody_reported: number;
  /// "ok" | "over_backed_drift" | "mismatch" — passed through raw rather than reduced to a
  /// boolean, because the middle one is ordinarily benign and a boolean would call it a failure.
  status: string;
};

export type Reserve = {
  /// False when this deployment has no treasury behind it; the section is then not rendered.
  configured: boolean;
  /// False when the treasury could not be reached. Never accompanied by stale figures.
  available?: boolean;
  error?: string;
  last_run?: ReserveRun | null;
  cache_age_seconds?: number;
};
