use crate::explorer::configuration::AppConfig;
use crate::explorer::models::{
    AccountDto, BlockDetailDto, BlockListItemDto, SearchResultDto, StatsDto, TransactionDetailDto,
    TransactionListItemDto, ValidatorDto,
};
use crate::explorer::node_client::{NodeClient, NodeClientError};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<ExplorerService>,
}

pub struct ExplorerService {
    node_client: NodeClient,
}

impl ExplorerService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            node_client: NodeClient::new(config.clutch_node_api_url, config.strict_mode),
        }
    }

    pub async fn get_blocks(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<BlockListItemDto>, NodeClientError> {
        self.node_client.latest_blocks(limit, offset).await
    }

    pub async fn get_block(&self, id: &str) -> Result<BlockDetailDto, NodeClientError> {
        self.node_client.block_by_id(id).await
    }

    pub async fn get_transactions(
        &self,
        limit: usize,
        offset: usize,
        address: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<TransactionListItemDto>, NodeClientError> {
        self.node_client
            .latest_transactions(limit, offset, address, status)
            .await
    }

    pub async fn get_transaction(
        &self,
        hash: &str,
    ) -> Result<TransactionDetailDto, NodeClientError> {
        self.node_client.transaction_by_hash(hash).await
    }

    pub async fn get_account(&self, address: &str) -> Result<AccountDto, NodeClientError> {
        self.node_client.account_by_address(address).await
    }

    pub async fn get_validators(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ValidatorDto>, NodeClientError> {
        self.node_client.validators(limit, offset).await
    }

    pub async fn get_stats(&self) -> Result<StatsDto, NodeClientError> {
        self.node_client.stats().await
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResultDto>, NodeClientError> {
        let q = query.trim();
        let mut results = Vec::new();
        if q.is_empty() {
            return Ok(results);
        }

        if q.starts_with("0xtx") {
            let tx = self.node_client.transaction_by_hash(q).await?;
            results.push(SearchResultDto {
                kind: "transaction".to_string(),
                identifier: tx.hash,
                summary: format!("Transaction in block {}", tx.block_height),
            });
        } else if q.starts_with("0x") {
            let account = self.node_client.account_by_address(q).await?;
            results.push(SearchResultDto {
                kind: "account".to_string(),
                identifier: account.address,
                summary: format!("Account with {} txs", account.tx_count),
            });
        } else {
            let block = self.node_client.block_by_id(q).await?;
            results.push(SearchResultDto {
                kind: "block".to_string(),
                identifier: block.hash,
                summary: format!("Block height {}", block.height),
            });
        }
        Ok(results)
    }
}
