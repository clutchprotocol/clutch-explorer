use crate::explorer::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use crate::explorer::models::{ApiErrorDto, ListResponseDto, PagingDto};
use crate::explorer::node_client::NodeClientError;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub status: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

pub async fn ready() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ready" })))
}

fn map_error(err: NodeClientError) -> (StatusCode, Json<ApiErrorDto>) {
    match err {
        NodeClientError::NotFound(message) => (
            StatusCode::NOT_FOUND,
            Json(ApiErrorDto {
                code: "not_found".to_string(),
                message,
            }),
        ),
        NodeClientError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            Json(ApiErrorDto {
                code: "invalid_request".to_string(),
                message,
            }),
        ),
        NodeClientError::Network(message) => (
            StatusCode::BAD_GATEWAY,
            Json(ApiErrorDto {
                code: "upstream_error".to_string(),
                message: message.to_string(),
            }),
        ),
    }
}

pub async fn list_blocks(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    match state.service.get_blocks(limit, offset).await {
        Ok(items) => (
            StatusCode::OK,
            Json(ListResponseDto {
                paging: PagingDto {
                    limit,
                    offset,
                    total: offset + items.len(),
                    has_more: items.len() == limit,
                },
                items,
            }),
        )
            .into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn get_block(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    if id.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiErrorDto {
                code: "invalid_request".to_string(),
                message: "block id must not be empty".to_string(),
            }),
        )
            .into_response();
    }
    match state.service.get_block(&id).await {
        Ok(block) => (StatusCode::OK, Json(json!(block))).into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn list_transactions(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    match state
        .service
        .get_transactions(
            limit,
            offset,
            query.address.as_deref(),
            query.status.as_deref(),
        )
        .await
    {
        Ok(items) => (
            StatusCode::OK,
            Json(ListResponseDto {
                paging: PagingDto {
                    limit,
                    offset,
                    total: offset + items.len(),
                    has_more: items.len() == limit,
                },
                items,
            }),
        )
            .into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    if hash.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiErrorDto {
                code: "invalid_request".to_string(),
                message: "transaction hash must not be empty".to_string(),
            }),
        )
            .into_response();
    }
    match state.service.get_transaction(&hash).await {
        Ok(tx) => (StatusCode::OK, Json(json!(tx))).into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn get_account(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    if address.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiErrorDto {
                code: "invalid_request".to_string(),
                message: "address must not be empty".to_string(),
            }),
        )
            .into_response();
    }
    match state.service.get_account(&address).await {
        Ok(account) => (StatusCode::OK, Json(json!(account))).into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn list_validators(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    match state.service.get_validators(limit, offset).await {
        Ok(items) => (
            StatusCode::OK,
            Json(ListResponseDto {
                paging: PagingDto {
                    limit,
                    offset,
                    total: offset + items.len(),
                    has_more: items.len() == limit,
                },
                items,
            }),
        )
            .into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.service.get_stats().await {
        Ok(stats) => (StatusCode::OK, Json(json!(stats))).into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    if query.q.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiErrorDto {
                code: "invalid_request".to_string(),
                message: "search query must not be empty".to_string(),
            }),
        )
            .into_response();
    }
    match state.service.search(&query.q).await {
        Ok(items) => (StatusCode::OK, Json(json!({ "items": items }))).into_response(),
        Err(err) => map_error(err).into_response(),
    }
}
