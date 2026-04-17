import { Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { AddressPage } from "./pages/AddressPage";
import { BlockDetailPage } from "./pages/BlockDetailPage";
import { BlocksPage } from "./pages/BlocksPage";
import { HomePage } from "./pages/HomePage";
import { TransactionDetailPage } from "./pages/TransactionDetailPage";
import { TransactionsPage } from "./pages/TransactionsPage";
import { ValidatorsPage } from "./pages/ValidatorsPage";

export function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<HomePage />} />
        <Route path="/blocks" element={<BlocksPage />} />
        <Route path="/blocks/:id" element={<BlockDetailPage />} />
        <Route path="/txs" element={<TransactionsPage />} />
        <Route path="/txs/:hash" element={<TransactionDetailPage />} />
        <Route path="/address/:address" element={<AddressPage />} />
        <Route path="/validators" element={<ValidatorsPage />} />
      </Route>
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
