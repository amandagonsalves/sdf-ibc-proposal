use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct BalanceResponse {
    balance: String,
}

#[tracing::instrument(skip(_state))]
pub async fn balance(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    tracing::info!(%address, "GET /balance/{address}");
    (
        StatusCode::OK,
        Json(BalanceResponse {
            balance: "0".to_string(),
        }),
    )
}
