use crate::explorer::error::ExplorerError;
use crate::explorer::models::{
    AccountDto, BlockDetailDto, BlockListItemDto, SearchResultDto, StatsDto, TransactionDetailDto,
    TransactionListItemDto, ValidatorDto,
};
use crate::explorer::node_client::{NodeClient, NodeClientError};
use crate::explorer::repository::{ExplorerRepository, RepoFuture};

pub struct NodeRepository {
    node_client: NodeClient,
}

impl NodeRepository {
    pub fn new(node_client: NodeClient) -> Self {
        Self { node_client }
    }
}

fn map_node_error(err: NodeClientError) -> ExplorerError {
    match err {
        NodeClientError::NotFound(msg) => ExplorerError::NotFound(msg),
        NodeClientError::InvalidRequest(msg) => ExplorerError::InvalidRequest(msg),
        NodeClientError::Network(err) => ExplorerError::Upstream(err.to_string()),
    }
}

impl ExplorerRepository for NodeRepository {
    fn get_blocks(&self, limit: usize, offset: usize) -> RepoFuture<'_, Vec<BlockListItemDto>> {
        Box::pin(async move {
            self.node_client
                .latest_blocks(limit, offset)
                .await
                .map_err(map_node_error)
        })
    }

    fn get_block(&self, id: String) -> RepoFuture<'_, BlockDetailDto> {
        Box::pin(async move { self.node_client.block_by_id(&id).await.map_err(map_node_error) })
    }

    fn get_transactions(
        &self,
        limit: usize,
        offset: usize,
        address: Option<String>,
        status: Option<String>,
    ) -> RepoFuture<'_, Vec<TransactionListItemDto>> {
        Box::pin(async move {
            self.node_client
                .latest_transactions(limit, offset, address.as_deref(), status.as_deref())
                .await
                .map_err(map_node_error)
        })
    }

    fn get_transaction(&self, hash: String) -> RepoFuture<'_, TransactionDetailDto> {
        Box::pin(async move {
            self.node_client
                .transaction_by_hash(&hash)
                .await
                .map_err(map_node_error)
        })
    }

    fn get_account(&self, address: String) -> RepoFuture<'_, AccountDto> {
        Box::pin(async move {
            self.node_client
                .account_by_address(&address)
                .await
                .map_err(map_node_error)
        })
    }

    fn get_validators(&self, limit: usize, offset: usize) -> RepoFuture<'_, Vec<ValidatorDto>> {
        Box::pin(async move {
            self.node_client
                .validators(limit, offset)
                .await
                .map_err(map_node_error)
        })
    }

    fn get_stats(&self) -> RepoFuture<'_, StatsDto> {
        Box::pin(async move { self.node_client.stats().await.map_err(map_node_error) })
    }

    fn search(&self, query: String) -> RepoFuture<'_, Vec<SearchResultDto>> {
        Box::pin(async move {
            let q = query.trim().to_string();
            if q.is_empty() {
                return Ok(Vec::new());
            }

            if q.starts_with("0xtx") {
                let tx = self
                    .node_client
                    .transaction_by_hash(&q)
                    .await
                    .map_err(map_node_error)?;
                return Ok(vec![SearchResultDto {
                    kind: "transaction".to_string(),
                    identifier: tx.hash,
                    summary: format!("Transaction in block {}", tx.block_height),
                }]);
            }

            if q.starts_with("0x") {
                let account = self
                    .node_client
                    .account_by_address(&q)
                    .await
                    .map_err(map_node_error)?;
                return Ok(vec![SearchResultDto {
                    kind: "account".to_string(),
                    identifier: account.address,
                    summary: format!("Account with {} txs", account.tx_count),
                }]);
            }

            let block = self
                .node_client
                .block_by_id(&q)
                .await
                .map_err(map_node_error)?;
            Ok(vec![SearchResultDto {
                kind: "block".to_string(),
                identifier: block.hash,
                summary: format!("Block height {}", block.height),
            }])
        })
    }
}
