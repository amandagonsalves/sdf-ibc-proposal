use std::sync::Arc;

mod types;

use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use soroban_client::xdr::{Limits, WriteXdr};
use types::SubmitSignedTxRequest;

fn err<E: std::fmt::Display>(status: StatusCode, e: E) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": e.to_string() })))
}

#[tracing::instrument(skip(state, req), fields(tx_bytes = req.tx_xdr.len()))]
pub async fn submit_signed_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubmitSignedTxRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!("POST /tx/submit");

    let tx_xdr = hex::decode(&req.tx_xdr)
        .map_err(|e| err(StatusCode::BAD_REQUEST, format!("tx_xdr hex: {e}")))?;

    let submitted = state
        .rpc
        .submit_and_wait_for_result(&tx_xdr)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "submit_and_wait_for_result failed");
            err(StatusCode::BAD_GATEWAY, e)
        })?;

    let return_value_xdr = match submitted.return_value {
        Some(value) => value
            .to_xdr(Limits::none())
            .map(hex::encode)
            .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("return_value XDR encode: {e}")))?,
        None => String::new(),
    };

    tracing::info!(hash = %submitted.hash, "tx submitted");

    Ok(Json(json!({
        "hash": submitted.hash,
        "return_value_xdr": return_value_xdr,
    })))
}
