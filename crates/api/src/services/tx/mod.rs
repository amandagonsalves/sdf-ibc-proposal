use std::sync::Arc;

mod types;

use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use types::{GetSignedTxResponse, GetUnsignedTxResponse, SignTxResponse, SubmitSignedTxResponse};

#[tracing::instrument(skip(_state))]
pub async fn submit_signed_tx(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    tracing::info!(%address, "POST /tx/submit");
    (
        StatusCode::OK,
        Json(SubmitSignedTxResponse {
            account_id: address,
        }),
    )
}

#[tracing::instrument(skip(_state))]
pub async fn sign_tx(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    tracing::info!(%address, "POST /tx/sign");
    (
        StatusCode::OK,
        Json(SignTxResponse {
            account_id: address,
        }),
    )
}

#[tracing::instrument(skip(_state))]
pub async fn get_unsigned_tx(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    tracing::info!(%address, "GET /tx/xdr");
    (
        StatusCode::OK,
        Json(GetUnsignedTxResponse {
            account_id: address,
        }),
    )
}

#[tracing::instrument(skip(_state))]
pub async fn get_signed_tx(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    tracing::info!(%address, "GET /tx/<tx_hash>");
    (
        StatusCode::OK,
        Json(GetSignedTxResponse {
            account_id: address,
        }),
    )
}
