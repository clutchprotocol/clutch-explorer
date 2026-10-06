import { useEffect, useState } from "react";
import { Link, NavLink, Outlet, useLocation } from "react-router-dom";
import { networkLabel } from "../utils/format";
import { SearchBox } from "./SearchBox";

type Theme = "light" | "dark";
const THEME_KEY = "clutch-explorer-theme";

function currentTheme(): Theme {
  const set = document.documentElement.dataset.theme;
  if (set === "light" || set === "dark") return set;
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function ThemeToggle() {
  const [theme, setTheme] = useState<Theme>(currentTheme);
  const next = theme === "dark" ? "light" : "dark";
  return (
    <button
      type="button"
      className="icon-button"
      aria-label={`Switch to ${next} theme`}
      title={`Switch to ${next} theme`}
      onClick={() => {
        document.documentElement.dataset.theme = next;
        try {
          localStorage.setItem(THEME_KEY, next);
        } catch {
          // Private windows can refuse storage; the theme still applies for this visit.
        }
        setTheme(next);
      }}
    >
      {theme === "dark" ? (
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <circle cx="8" cy="8" r="3" fill="none" stroke="currentColor" strokeWidth="1.5" />
          <path
            d="M8 1v2M8 13v2M1 8h2M13 8h2M3 3l1.4 1.4M11.6 11.6L13 13M3 13l1.4-1.4M11.6 4.4L13 3"
            stroke="currentColor"
            strokeWidth="1.5"
          />
        </svg>
      ) : (
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M13 9.5A5.5 5.5 0 0 1 6.5 3a5.5 5.5 0 1 0 6.5 6.5z" fill="none" stroke="currentColor" strokeWidth="1.5" />
        </svg>
      )}
    </button>
  );
}

const NAV = [
  { to: "/", label: "Overview", end: true },
  { to: "/blocks", label: "Blocks" },
  { to: "/txs", label: "Transactions" },
  { to: "/validators", label: "Validators" },
];

export function Layout() {
  const [menuOpen, setMenuOpen] = useState(false);
  const location = useLocation();
  const network = networkLabel();
  const isHome = location.pathname === "/";

  useEffect(() => setMenuOpen(false), [location.pathname, location.search]);

  return (
    <div className="app-shell">
      <a className="skip-link" href="#main">
        Skip to content
      </a>
      <header className="topbar">
        <div className="topbar-inner">
          <Link to="/" className="brand" aria-label="Clutch Explorer home">
            <img src="/favicon.svg" alt="" width="28" height="28" />
            <span>
              Clutch <em>Explorer</em>
            </span>
          </Link>
          <span className={`network-chip network-chip--${network.tone}`}>{network.label}</span>

          <nav id="primary-nav" className={`topnav${menuOpen ? " is-open" : ""}`}>
            {NAV.map((item) => (
              <NavLink key={item.to} to={item.to} end={item.end}>
                {item.label}
              </NavLink>
            ))}
          </nav>

          <div className="topbar-actions">
            <ThemeToggle />
            <button
              type="button"
              className="icon-button menu-button"
              aria-expanded={menuOpen}
              aria-controls="primary-nav"
              aria-label="Menu"
              onClick={() => setMenuOpen((open) => !open)}
            >
              <svg viewBox="0 0 16 16" aria-hidden="true">
                {menuOpen ? (
                  <path d="M3 3l10 10M13 3L3 13" stroke="currentColor" strokeWidth="1.6" />
                ) : (
                  <path d="M2 4h12M2 8h12M2 12h12" stroke="currentColor" strokeWidth="1.6" />
                )}
              </svg>
            </button>
          </div>
        </div>
        {isHome ? null : (
          <div className="topbar-search">
            <SearchBox hotkey />
          </div>
        )}
      </header>

      <main id="main" className="page-content">
        <Outlet />
      </main>

      <footer className="footer">
        <div className="footer-inner">
          <span>
            <strong>Clutch Protocol</strong> · alpha software. Figures come from this explorer's own
            index of the chain.
          </span>
          <nav aria-label="Elsewhere">
            <a href="https://docs.clutchprotocol.io/clutch-explorer/overview" target="_blank" rel="noopener">
              Docs
            </a>
            <a href="https://docs.clutchprotocol.io/clutch-explorer/api-reference" target="_blank" rel="noopener">
              API
            </a>
            <a href="https://clutchprotocol.io" target="_blank" rel="noopener">
              clutchprotocol.io
            </a>
            <a href="https://github.com/clutchprotocol/clutch-explorer" target="_blank" rel="noopener">
              GitHub
            </a>
          </nav>
        </div>
      </footer>
    </div>
  );
}
