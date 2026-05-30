use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AppState;

const BASE_FEE: u32 = 1_000;

#[derive(Deserialize)]
pub struct PrepareRequest {
    pub method: String,
    #[serde(default)]
    pub args_xdr: Vec<String>,
    pub source_account: String,
}

#[derive(Serialize)]
pub struct PrepareResponse {
    pub tx_xdr: String,
}

fn err<E: std::fmt::Display>(status: StatusCode, e: E) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": e.to_string() })))
}

#[tracing::instrument(skip(state, req), fields(method = %req.method, source = %req.source_account))]
pub async fn prepare_invoke(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PrepareRequest>,
) -> Result<Json<PrepareResponse>, (StatusCode, Json<Value>)> {
    tracing::info!("POST /contract/prepare");

    if state.ibc_contract_id.is_empty() {
        return Err(err(StatusCode::BAD_GATEWAY, "IBC_CONTRACT_ID not configured"));
    }

    let mut args_xdr = Vec::with_capacity(req.args_xdr.len());
    for (i, a) in req.args_xdr.iter().enumerate() {
        let bytes = hex::decode(a)
            .map_err(|e| err(StatusCode::BAD_REQUEST, format!("args_xdr[{i}] hex: {e}")))?;
        args_xdr.push(bytes);
    }

    let tx_xdr = state
        .rpc
        .build_unsigned_router_invoke(
            &state.ibc_contract_id,
            &req.method,
            &args_xdr,
            &req.source_account,
            &state.network_passphrase,
            BASE_FEE,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "build_unsigned_router_invoke failed");
            err(StatusCode::BAD_GATEWAY, e)
        })?;

    Ok(Json(PrepareResponse {
        tx_xdr: hex::encode(tx_xdr),
    }))
}
