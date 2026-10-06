use crate::explorer::app::AppState;
use crate::explorer::error::ExplorerError;
use crate::explorer::models::{
    AccountActivityDto, AccountDto, BlockDetailDto, BlockListItemDto, ListResponseDto, PagingDto,
    StatsDto, TransactionDetailDto, TransactionFilter, TransactionListItemDto, ValidatorDto,
};
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

type ApiResult<T> = Result<Json<T>, ExplorerError>;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub status: Option<String>,
    pub address: Option<String>,
    pub block: Option<u64>,
    #[serde(rename = "type")]
    pub tx_type: Option<String>,
}

impl ListQuery {
    /// `(limit, offset)`: 20 by default, never more than 100.
    fn page(&self) -> (usize, usize) {
        (self.limit.unwrap_or(20).min(100), self.offset.unwrap_or(0))
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// The paging block is derived from the page itself: the API never counts the whole table.
fn list<T: Serialize>(items: Vec<T>, limit: usize, offset: usize) -> ListResponseDto<T> {
    ListResponseDto {
        paging: PagingDto {
            limit,
            offset,
            total: offset + items.len(),
            has_more: items.len() == limit,
        },
        items,
    }
}

fn require(value: &str, what: &str) -> Result<(), ExplorerError> {
    if value.trim().is_empty() {
        return Err(ExplorerError::InvalidRequest(format!(
            "{what} must not be empty"
        )));
    }
    Ok(())
}

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn ready() -> Json<Value> {
    Json(json!({ "status": "ready" }))
}

pub async fn list_blocks(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<ListResponseDto<BlockListItemDto>> {
    let (limit, offset) = query.page();
    let items = state.repo.get_blocks(limit, offset).await?;
    Ok(Json(list(items, limit, offset)))
}

pub async fn get_block(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<BlockDetailDto> {
    require(&id, "block id")?;
    Ok(Json(state.repo.get_block(&id).await?))
}

pub async fn list_transactions(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<ListResponseDto<TransactionListItemDto>> {
    let (limit, offset) = query.page();
    let filter = TransactionFilter {
        address: query.address,
        status: query.status,
        block: query.block,
        tx_type: query.tx_type,
    };
    let items = state.repo.get_transactions(limit, offset, filter).await?;
    Ok(Json(list(items, limit, offset)))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> ApiResult<TransactionDetailDto> {
    require(&hash, "transaction hash")?;
    Ok(Json(state.repo.get_transaction(&hash).await?))
}

pub async fn get_account(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<AccountDto> {
    require(&address, "address")?;
    Ok(Json(state.repo.get_account(&address).await?))
}

pub async fn get_account_activity(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(query): Query<ListQuery>,
) -> ApiResult<ListResponseDto<AccountActivityDto>> {
    require(&address, "address")?;
    let (limit, offset) = query.page();
    let items = state
        .repo
        .get_account_activity(&address, limit, offset)
        .await?;
    Ok(Json(list(items, limit, offset)))
}

pub async fn list_validators(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<ListResponseDto<ValidatorDto>> {
    let (limit, offset) = query.page();
    let items = state.repo.get_validators(limit, offset).await?;
    Ok(Json(list(items, limit, offset)))
}

pub async fn get_stats(State(state): State<AppState>) -> ApiResult<StatsDto> {
    Ok(Json(state.repo.get_stats().await?))
}

/// The reserve position behind CLT: supply, liability and custody from a single reconciliation
/// run, with that run's timestamp and the age of this answer.
///
/// Always 200. "The treasury is unreachable" and "no run has happened yet" are both states a
/// reader needs to see, and a status code would push a consumer into an error branch that hides
/// which one it is.
pub async fn get_reserve(State(state): State<AppState>) -> Json<Value> {
    Json(state.reserve.latest().await)
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Value> {
    require(&query.q, "search query")?;
    let items = state.repo.search(&query.q).await?;
    Ok(Json(json!({ "items": items })))
}
