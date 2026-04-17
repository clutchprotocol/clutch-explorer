use crate::explorer::configuration::AppConfig;
use crate::explorer::error::ExplorerError;
use crate::explorer::models::{
    AccountDto, BlockDetailDto, BlockListItemDto, SearchResultDto, StatsDto, TransactionDetailDto,
    TransactionListItemDto, ValidatorDto,
};
use crate::explorer::node_client::NodeClient;
use crate::explorer::node_repository::NodeRepository;
use crate::explorer::postgres_repository::PostgresRepository;
use crate::explorer::repository::ExplorerRepository;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<ExplorerService>,
}

pub struct ExplorerService {
    repository: Arc<dyn ExplorerRepository>,
}

impl ExplorerService {
    pub fn new(config: AppConfig, pg_pool: Option<PgPool>) -> Result<Self, ExplorerError> {
        let repository: Arc<dyn ExplorerRepository> = match config.data_source.as_str() {
            "postgres" => {
                let pool = pg_pool
                    .ok_or_else(|| ExplorerError::InvalidRequest("postgres pool missing".to_string()))?;
                Arc::new(PostgresRepository::new(pool))
            }
            "node" => Arc::new(NodeRepository::new(NodeClient::new(
                config.clutch_node_api_url,
                config.strict_mode,
            ))),
            other => {
                return Err(ExplorerError::InvalidRequest(format!(
                    "unsupported data_source: {}",
                    other
                )))
            }
        };

        Ok(Self { repository })
    }

    pub fn from_repository(repository: Arc<dyn ExplorerRepository>) -> Self {
        Self {
            repository,
        }
    }

    pub async fn get_blocks(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<BlockListItemDto>, ExplorerError> {
        self.repository.get_blocks(limit, offset).await
    }

    pub async fn get_block(&self, id: &str) -> Result<BlockDetailDto, ExplorerError> {
        self.repository.get_block(id.to_string()).await
    }

    pub async fn get_transactions(
        &self,
        limit: usize,
        offset: usize,
        address: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<TransactionListItemDto>, ExplorerError> {
        self.repository
            .get_transactions(
                limit,
                offset,
                address.map(ToString::to_string),
                status.map(ToString::to_string),
            )
            .await
    }

    pub async fn get_transaction(
        &self,
        hash: &str,
    ) -> Result<TransactionDetailDto, ExplorerError> {
        self.repository.get_transaction(hash.to_string()).await
    }

    pub async fn get_account(&self, address: &str) -> Result<AccountDto, ExplorerError> {
        self.repository.get_account(address.to_string()).await
    }

    pub async fn get_validators(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ValidatorDto>, ExplorerError> {
        self.repository.get_validators(limit, offset).await
    }

    pub async fn get_stats(&self) -> Result<StatsDto, ExplorerError> {
        self.repository.get_stats().await
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResultDto>, ExplorerError> {
        self.repository.search(query.to_string()).await
    }
}
