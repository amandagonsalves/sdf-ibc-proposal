use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize, Debug, Default)]
pub struct EventsQuery {
    pub contract_id: Option<String>,
    pub cursor: Option<String>,
    pub start_ledger: Option<u32>,
    pub limit: Option<u32>,
}

#[tracing::instrument(skip(state))]
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventsQuery>,
) -> impl IntoResponse {
    tracing::info!(
        contract_id = ?params.contract_id,
        cursor = ?params.cursor,
        start_ledger = ?params.start_ledger,
        limit = ?params.limit,
        "GET /events"
    );

    let latest = match state.rpc.latest_ledger_sequence().await {
        Ok(seq) => seq,
        Err(error) => {
            tracing::error!(%error, "latest_ledger_sequence failed in /events fallback");
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };

    static WARNED_STUB: AtomicBool = AtomicBool::new(false);
    if !WARNED_STUB.swap(true, Ordering::Relaxed) {
        tracing::warn!(
            "/events is a stub — returning empty event pages until Soroban getEvents is wired through `RpcClient`."
        );
    }

    let body = json!({
        "latest_ledger": latest,
        "cursor": params.cursor.unwrap_or_default(),
        "events": [],
    });

    (StatusCode::OK, Json(body)).into_response()
}
