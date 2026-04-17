use crate::explorer::error::ExplorerError;
use crate::explorer::models::{
    AccountDto, BlockDetailDto, BlockListItemDto, SearchResultDto, StatsDto, TransactionDetailDto,
    TransactionListItemDto, ValidatorDto,
};
use std::future::Future;
use std::pin::Pin;

pub type RepoFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ExplorerError>> + Send + 'a>>;

pub trait ExplorerRepository: Send + Sync {
    fn get_blocks(&self, limit: usize, offset: usize) -> RepoFuture<'_, Vec<BlockListItemDto>>;
    fn get_block(&self, id: String) -> RepoFuture<'_, BlockDetailDto>;
    fn get_transactions(
        &self,
        limit: usize,
        offset: usize,
        address: Option<String>,
        status: Option<String>,
    ) -> RepoFuture<'_, Vec<TransactionListItemDto>>;
    fn get_transaction(&self, hash: String) -> RepoFuture<'_, TransactionDetailDto>;
    fn get_account(&self, address: String) -> RepoFuture<'_, AccountDto>;
    fn get_validators(&self, limit: usize, offset: usize) -> RepoFuture<'_, Vec<ValidatorDto>>;
    fn get_stats(&self) -> RepoFuture<'_, StatsDto>;
    fn search(&self, query: String) -> RepoFuture<'_, Vec<SearchResultDto>>;
}
