use crate::explorer::configuration::AppConfig;
use crate::explorer::models::{AccountDto, BlockDto, SearchResultDto, StatsDto, TransactionDto, ValidatorDto};
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
            node_client: NodeClient::new(config.clutch_node_api_url),
        }
    }

    pub async fn get_blocks(&self, limit: usize) -> Result<Vec<BlockDto>, NodeClientError> {
        self.node_client.latest_blocks(limit).await
    }

    pub async fn get_block(&self, id: &str) -> Result<BlockDto, NodeClientError> {
        self.node_client.block_by_id(id).await
    }

    pub async fn get_transactions(
        &self,
        limit: usize,
    ) -> Result<Vec<TransactionDto>, NodeClientError> {
        self.node_client.latest_transactions(limit).await
    }

    pub async fn get_transaction(&self, hash: &str) -> Result<TransactionDto, NodeClientError> {
        self.node_client.transaction_by_hash(hash).await
    }

    pub async fn get_account(&self, address: &str) -> Result<AccountDto, NodeClientError> {
        self.node_client.account_by_address(address).await
    }

    pub async fn get_validators(&self) -> Result<Vec<ValidatorDto>, NodeClientError> {
        self.node_client.validators().await
    }

    pub async fn get_stats(&self) -> Result<StatsDto, NodeClientError> {
        self.node_client.stats().await
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResultDto>, NodeClientError> {
        let mut results = Vec::new();
        if query.starts_with("0xtx") {
            results.push(SearchResultDto {
                kind: "transaction".to_string(),
                identifier: query.to_string(),
                summary: "Transaction hash match".to_string(),
            });
        } else if query.starts_with("0x") {
            results.push(SearchResultDto {
                kind: "account".to_string(),
                identifier: query.to_string(),
                summary: "Account address match".to_string(),
            });
        } else {
            results.push(SearchResultDto {
                kind: "block".to_string(),
                identifier: query.to_string(),
                summary: "Block height/hash match".to_string(),
            });
        }
        Ok(results)
    }
}
