import { FormEvent, useState } from "react";
import { Link, NavLink, Outlet, useNavigate } from "react-router-dom";

function resolveSearchTarget(value: string): string {
  if (value.startsWith("0xtx")) return `/txs/${encodeURIComponent(value)}`;
  if (value.startsWith("0x")) return `/address/${encodeURIComponent(value)}`;
  return `/blocks/${encodeURIComponent(value)}`;
}

export function Layout() {
  const [query, setQuery] = useState("");
  const navigate = useNavigate();

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    const trimmed = query.trim();
    if (!trimmed) return;
    navigate(resolveSearchTarget(trimmed));
  };

  return (
    <div className="app-shell">
      <header className="topbar">
        <Link to="/" className="brand">
          Clutch Explorer
        </Link>
        <nav className="topnav">
          <NavLink to="/" end>
            Home
          </NavLink>
          <NavLink to="/blocks">Blocks</NavLink>
          <NavLink to="/txs">Transactions</NavLink>
          <NavLink to="/validators">Validators</NavLink>
        </nav>
        <form className="search" onSubmit={onSubmit}>
          <input
            placeholder="Search by block, tx, or address"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
          <button type="submit">Go</button>
        </form>
      </header>
      <main className="page-content">
        <Outlet />
      </main>
    </div>
  );
}
