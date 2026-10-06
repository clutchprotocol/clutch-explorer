import type {
  Account,
  AccountActivity,
  BlockDetail,
  BlockListItem,
  ListResponse,
  Reserve,
  SearchResult,
  Stats,
  TransactionDetail,
  TransactionListItem,
  Validator,
} from "./types";

const RAW_API_BASE =
  import.meta.env.VITE_EXPLORER_API_URL ??
  (typeof window !== "undefined" && window.location.hostname === "localhost"
    ? "http://localhost:8088"
    : "/api");

const API_BASE = RAW_API_BASE.endsWith("/api")
  ? RAW_API_BASE
  : `${RAW_API_BASE.replace(/\/+$/, "")}/api`;

/** Carries the HTTP status so a page can tell "not found" from "the explorer is down". */
export class ApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }

  get isNotFound() {
    return this.status === 404;
  }
}

async function api<T>(path: string): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`);
  } catch {
    throw new ApiError("The explorer API could not be reached.", 0);
  }
  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    const message =
      typeof payload?.message === "string"
        ? payload.message
        : `Request failed with ${response.status}`;
    throw new ApiError(message, response.status);
  }
  return response.json();
}

export type TransactionFilter = {
  address?: string;
  status?: string;
  block?: number;
  /** A `function_call_type`, e.g. `Transfer` or `RidePay`. */
  type?: string;
};

export const explorerApi = {
  getStats: () => api<Stats>("/v1/stats"),
  getReserve: () => api<Reserve>("/v1/reserve"),
  getBlocks: (limit = 20, offset = 0) =>
    api<ListResponse<BlockListItem>>(`/v1/blocks?limit=${limit}&offset=${offset}`),
  getBlockById: (id: string) => api<BlockDetail>(`/v1/blocks/${encodeURIComponent(id)}`),
  getTransactions: (limit = 20, offset = 0, filter: TransactionFilter = {}) => {
    const params = new URLSearchParams({
      limit: String(limit),
      offset: String(offset),
    });
    if (filter.address) params.set("address", filter.address);
    if (filter.status) params.set("status", filter.status);
    if (filter.block !== undefined) params.set("block", String(filter.block));
    if (filter.type) params.set("type", filter.type);
    return api<ListResponse<TransactionListItem>>(`/v1/transactions?${params.toString()}`);
  },
  getTransactionByHash: (hash: string) =>
    api<TransactionDetail>(`/v1/transactions/${encodeURIComponent(hash)}`),
  getAccountByAddress: (address: string) =>
    api<Account>(`/v1/accounts/${encodeURIComponent(address)}`),
  getAccountActivity: (address: string, limit = 20, offset = 0) =>
    api<ListResponse<AccountActivity>>(
      `/v1/accounts/${encodeURIComponent(address)}/activity?limit=${limit}&offset=${offset}`,
    ),
  getValidators: (limit = 20, offset = 0) =>
    api<ListResponse<Validator>>(`/v1/validators?limit=${limit}&offset=${offset}`),
  search: (query: string) =>
    api<{ items: SearchResult[] }>(`/v1/search?q=${encodeURIComponent(query)}`),
};
