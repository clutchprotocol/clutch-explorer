import { useEffect, useState } from "react";

type JsonValue = Record<string, unknown>;

const API_BASE = import.meta.env.VITE_EXPLORER_API_URL ?? "http://localhost:8088";

async function getJson(path: string): Promise<JsonValue> {
  const response = await fetch(`${API_BASE}${path}`);
  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }
  return response.json();
}

export function App() {
  const [stats, setStats] = useState<JsonValue>({});
  const [blocks, setBlocks] = useState<JsonValue[]>([]);
  const [transactions, setTransactions] = useState<JsonValue[]>([]);
  const [validators, setValidators] = useState<JsonValue[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<JsonValue[]>([]);
  const [account, setAccount] = useState<JsonValue | null>(null);
  const [accountInput, setAccountInput] = useState("0xabc");
  const [error, setError] = useState("");

  const refresh = async () => {
    try {
      setError("");
      const [statsData, blocksData, txData, validatorsData] = await Promise.all([
        getJson("/api/v1/stats"),
        getJson("/api/v1/blocks?limit=10"),
        getJson("/api/v1/transactions?limit=10"),
        getJson("/api/v1/validators"),
      ]);
      setStats(statsData);
      setBlocks((blocksData.items as JsonValue[]) ?? []);
      setTransactions((txData.items as JsonValue[]) ?? []);
      setValidators((validatorsData.items as JsonValue[]) ?? []);
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const runSearch = async () => {
    if (!searchQuery.trim()) return;
    try {
      const payload = await getJson(`/api/v1/search?q=${encodeURIComponent(searchQuery)}`);
      setSearchResults((payload.items as JsonValue[]) ?? []);
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const loadAccount = async () => {
    if (!accountInput.trim()) return;
    try {
      const payload = await getJson(`/api/v1/accounts/${encodeURIComponent(accountInput)}`);
      setAccount(payload);
    } catch (err) {
      setError((err as Error).message);
    }
  };

  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, 10000);
    return () => clearInterval(timer);
  }, []);

  return (
    <main className="container">
      <h1>Clutch Explorer</h1>
      <p className="subtitle">Node-connected blockchain explorer dashboard</p>
      {error ? <p className="error">{error}</p> : null}

      <section>
        <h2>Chain Stats</h2>
        <pre>{JSON.stringify(stats, null, 2)}</pre>
      </section>

      <section>
        <h2>Search</h2>
        <div className="row">
          <input value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} />
          <button onClick={runSearch}>Search</button>
        </div>
        <pre>{JSON.stringify(searchResults, null, 2)}</pre>
      </section>

      <section>
        <h2>Account Lookup</h2>
        <div className="row">
          <input value={accountInput} onChange={(e) => setAccountInput(e.target.value)} />
          <button onClick={loadAccount}>Load</button>
        </div>
        <pre>{JSON.stringify(account, null, 2)}</pre>
      </section>

      <section>
        <h2>Latest Blocks</h2>
        <pre>{JSON.stringify(blocks, null, 2)}</pre>
      </section>

      <section>
        <h2>Latest Transactions</h2>
        <pre>{JSON.stringify(transactions, null, 2)}</pre>
      </section>

      <section>
        <h2>Validators</h2>
        <pre>{JSON.stringify(validators, null, 2)}</pre>
      </section>
    </main>
  );
}
