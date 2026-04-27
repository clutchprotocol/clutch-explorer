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
  timestamp: string;
};

export type TransactionDetail = TransactionListItem & {
  nonce: number;
  tx_index: number;
};

export type Account = {
  address: string;
  balance: number;
  nonce: number;
  tx_count: number;
  is_contract: boolean;
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
