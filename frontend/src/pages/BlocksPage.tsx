import { explorerApi } from "../api/client";
import { BlocksTable } from "../components/tables";
import { ErrorBanner, PageHeader, Pagination, Panel, Skeleton } from "../components/ui";
import { useApi } from "../hooks/useApi";
import { usePage } from "../hooks/usePage";

const PAGE_SIZE = 25;

export function BlocksPage() {
  const [page, setPage] = usePage();
  const { data, error } = useApi(() => explorerApi.getBlocks(PAGE_SIZE, (page - 1) * PAGE_SIZE), [page], page === 1 ? 10_000 : undefined);

  return (
    <div className="page-grid">
      <PageHeader eyebrow="Chain" title="Blocks">
        <p className="lede">Newest first. Validators take turns producing blocks in fixed slots.</p>
      </PageHeader>
      <ErrorBanner message={error?.message} />
      <Panel flush>
        {data ? <BlocksTable blocks={data.items} /> : error ? null : <Skeleton rows={12} />}
        <Pagination page={page} hasMore={Boolean(data?.paging.has_more)} onChange={setPage} />
      </Panel>
    </div>
  );
}
