import type {
  Account,
  BlockDetail,
  BlockListItem,
  ListResponse,
  SearchResult,
  Stats,
  TransactionDetail,
  TransactionListItem,
  Validator,
} from "./types";

const API_BASE =
  import.meta.env.VITE_EXPLORER_API_URL ??
  (typeof window !== "undefined" && window.location.hostname === "localhost"
    ? "http://localhost:8088"
    : "/api");

async function api<T>(path: string): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`);
  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    const message =
      typeof payload?.message === "string"
        ? payload.message
        : `Request failed with ${response.status}`;
    throw new Error(message);
  }
  return response.json();
}

export const explorerApi = {
  getStats: () => api<Stats>("/api/v1/stats"),
  getBlocks: (limit = 20, offset = 0) =>
    api<ListResponse<BlockListItem>>(`/api/v1/blocks?limit=${limit}&offset=${offset}`),
  getBlockById: (id: string) => api<BlockDetail>(`/api/v1/blocks/${encodeURIComponent(id)}`),
  getTransactions: (limit = 20, offset = 0, address?: string, status?: string) => {
    const params = new URLSearchParams({
      limit: String(limit),
      offset: String(offset),
    });
    if (address) params.set("address", address);
    if (status) params.set("status", status);
    return api<ListResponse<TransactionListItem>>(`/api/v1/transactions?${params.toString()}`);
  },
  getTransactionByHash: (hash: string) =>
    api<TransactionDetail>(`/api/v1/transactions/${encodeURIComponent(hash)}`),
  getAccountByAddress: (address: string) =>
    api<Account>(`/api/v1/accounts/${encodeURIComponent(address)}`),
  getValidators: (limit = 20, offset = 0) =>
    api<ListResponse<Validator>>(`/api/v1/validators?limit=${limit}&offset=${offset}`),
  search: (query: string) =>
    api<{ items: SearchResult[] }>(`/api/v1/search?q=${encodeURIComponent(query)}`),
};
