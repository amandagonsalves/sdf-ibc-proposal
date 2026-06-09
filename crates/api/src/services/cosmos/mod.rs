//! Cosmos-chain proxy and signing service.
//!
//! Read endpoints forward to the configured `COSMOS_REST_URL`. Write endpoints
//! build, sign, and broadcast Cosmos SDK txs using keys loaded from
//! `COSMOS_PROPOSER_PRIVATE_KEY` and `COSMOS_FUNDER_PRIVATE_KEY`.
//!
//! The shared transport + signing logic lives in [`client::CosmosClient`].

pub mod client;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::{IntoParams, ToSchema};

use crate::AppState;

use self::client::CosmosClient;

fn err<E: std::fmt::Display>(status: StatusCode, e: E) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": e.to_string() })))
}

fn bad_gateway<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    err(StatusCode::BAD_GATEWAY, e)
}

fn bad_request<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    err(StatusCode::BAD_REQUEST, e)
}

/// `GET /cosmos/node-info` — proxy for
/// `/cosmos/base/tendermint/v1beta1/node_info` on the configured Cosmos REST
/// endpoint. Useful as a chain reachability probe.
///
/// Returns the upstream JSON verbatim; `502` if the chain is unreachable.
#[utoipa::path(
    get,
    path = "/cosmos/node-info",
    tag = "Cosmos read",
    responses(
        (status = 200, description = "Upstream Tendermint node_info"),
        (status = 502, description = "Cosmos REST unreachable"),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn node_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("GET /cosmos/node-info");
    match state.cosmos.node_info().await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            tracing::error!(error = %e, "node_info failed");
            Err(bad_gateway(e))
        }
    }
}

/// Query parameters for [`proposals`].
#[derive(Deserialize, IntoParams)]
pub struct ProposalsQuery {
    /// Proposal status filter. Accepts short forms (`voting`, `deposit`,
    /// `passed`, `rejected`, `failed`) or full SDK enum names
    /// (`PROPOSAL_STATUS_VOTING_PERIOD`, etc.). Defaults to voting period.
    pub status: Option<String>,
}

fn proposal_status_str(status: &str) -> String {
    let lower = status.to_ascii_lowercase();
    match lower.as_str() {
        "voting" | "voting_period" => "PROPOSAL_STATUS_VOTING_PERIOD".to_string(),
        "deposit" | "deposit_period" => "PROPOSAL_STATUS_DEPOSIT_PERIOD".to_string(),
        "passed" => "PROPOSAL_STATUS_PASSED".to_string(),
        "rejected" => "PROPOSAL_STATUS_REJECTED".to_string(),
        "failed" => "PROPOSAL_STATUS_FAILED".to_string(),
        _ if status.starts_with("PROPOSAL_STATUS_") => status.to_string(),
        _ => "PROPOSAL_STATUS_UNSPECIFIED".to_string(),
    }
}

/// `GET /cosmos/gov/proposals` — list gov proposals filtered by status.
///
/// Query: see [`ProposalsQuery`]. Forwards to
/// `/cosmos/gov/v1/proposals?proposal_status=…` on the upstream chain.
#[utoipa::path(
    get,
    path = "/cosmos/gov/proposals",
    tag = "Cosmos read",
    params(ProposalsQuery),
    responses(
        (status = 200, description = "Proposals matching the requested status"),
        (status = 502, description = "Cosmos REST unreachable"),
    )
)]
#[tracing::instrument(skip(state, q))]
pub async fn proposals(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ProposalsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let status = q
        .status
        .as_deref()
        .map(proposal_status_str)
        .unwrap_or_else(|| "PROPOSAL_STATUS_VOTING_PERIOD".to_string());
    tracing::debug!(%status, "GET /cosmos/gov/proposals");
    match state.cosmos.proposals_by_status(&status).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            tracing::error!(error = %e, %status, "proposals_by_status failed");
            Err(bad_gateway(e))
        }
    }
}

/// `GET /cosmos/gov/proposals/{id}` — fetch a single gov proposal by id.
///
/// Forwards to `/cosmos/gov/v1/proposals/{id}` on the upstream chain.
#[utoipa::path(
    get,
    path = "/cosmos/gov/proposals/{id}",
    tag = "Cosmos read",
    params(
        ("id" = u64, Path, description = "Proposal id"),
    ),
    responses(
        (status = 200, description = "Proposal record"),
        (status = 502, description = "Cosmos REST unreachable"),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn proposal_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!(proposal_id = id, "GET /cosmos/gov/proposals/{id}");
    match state.cosmos.proposal(id).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            tracing::error!(error = %e, proposal_id = id, "proposal lookup failed");
            Err(bad_gateway(e))
        }
    }
}

/// `GET /cosmos/gov/params/deposit` — current gov deposit parameters
/// (`min_deposit`, `max_deposit_period`, `min_initial_deposit_ratio`, etc.).
#[utoipa::path(
    get,
    path = "/cosmos/gov/params/deposit",
    tag = "Cosmos read",
    responses(
        (status = 200, description = "Gov deposit params"),
        (status = 502, description = "Cosmos REST unreachable"),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn gov_deposit_params(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("GET /cosmos/gov/params/deposit");
    match state.cosmos.gov_deposit_params().await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            tracing::error!(error = %e, "gov_deposit_params failed");
            Err(bad_gateway(e))
        }
    }
}

/// `GET /cosmos/tx/{hash}` — fetch a Cosmos tx by hash.
///
/// Returns `404` if the chain doesn't know the hash yet (in-flight, never
/// landed, or pruned). Returns the upstream JSON verbatim otherwise — the
/// landed `tx_response.code` is what indicates actual on-chain success.
#[utoipa::path(
    get,
    path = "/cosmos/tx/{hash}",
    tag = "Cosmos read",
    params(
        ("hash" = String, Path, description = "Cosmos tx hash (hex, no 0x prefix)"),
    ),
    responses(
        (status = 200, description = "Tx record (check tx_response.code for on-chain success)"),
        (status = 404, description = "Tx not yet known to the chain"),
        (status = 502, description = "Cosmos REST unreachable"),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn tx_by_hash(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!(%hash, "GET /cosmos/tx/{hash}");
    match state.cosmos.tx_by_hash(&hash).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            let not_found = e.to_string().contains("not found");
            if not_found {
                tracing::debug!(%hash, "tx not yet found");
                Err(err(StatusCode::NOT_FOUND, e))
            } else {
                tracing::error!(error = %e, %hash, "tx_by_hash failed");
                Err(bad_gateway(e))
            }
        }
    }
}

/// `GET /cosmos/ibc-wasm/checksums` — list every wasm checksum registered with
/// the chain's 08-wasm light client module.
#[utoipa::path(
    get,
    path = "/cosmos/ibc-wasm/checksums",
    tag = "Cosmos read",
    responses(
        (status = 200, description = "List of registered wasm checksums"),
        (status = 502, description = "Cosmos REST unreachable"),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn ibc_wasm_checksums(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("GET /cosmos/ibc-wasm/checksums");
    match state.cosmos.ibc_wasm_checksums().await {
        Ok(v) => {
            let count = v
                .get("checksums")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            tracing::info!(count, "[cosmos] 08-wasm checksums");
            Ok(Json(v))
        }
        Err(e) => {
            tracing::error!(error = %e, "ibc_wasm_checksums failed");
            Err(bad_gateway(e))
        }
    }
}

/// Request body for [`submit_store_code`].
#[derive(Deserialize, ToSchema)]
pub struct StoreCodeRequest {
    /// The wasm bytecode, base64-encoded.
    pub wasm_base64: String,
    /// Proposal title.
    pub title: String,
    /// Proposal summary / description.
    pub summary: String,
    /// Initial deposit in the chain's gas denom (e.g. `uosmo`).
    pub deposit_amount: u128,
    /// Gas limit for the submit-proposal tx.
    pub gas_limit: u64,
    /// Tx fee in the chain's gas denom.
    pub fee_amount: u128,
    /// When true, the handler polls until the tx lands in a block and reports
    /// the on-chain `code` plus the extracted `proposal_id`. Defaults to false.
    #[serde(default)]
    pub wait_for_landing: bool,
    /// Polling deadline when `wait_for_landing` is set. Defaults to 30s.
    #[serde(default = "default_wait_secs")]
    pub wait_timeout_secs: u64,
}

fn default_wait_secs() -> u64 {
    30
}

/// `POST /cosmos/ibc-wasm/store-code` — submit a `MsgSubmitProposal` wrapping
/// `ibc.lightclients.wasm.v1.MsgStoreCode`, signed by the proposer key.
///
/// Body: [`StoreCodeRequest`]. The proposer (and only the proposer) needs to
/// hold the deposit + fee balance.
///
/// Returns `{ tx_hash, code, raw_log }` on broadcast success. If
/// `wait_for_landing` is true the response also includes the full
/// `tx_response` and the extracted `proposal_id`, and an on-chain failure
/// (DeliverTx `code != 0`) maps to `502`.
#[utoipa::path(
    post,
    path = "/cosmos/ibc-wasm/store-code",
    tag = "Cosmos write",
    request_body = StoreCodeRequest,
    responses(
        (status = 200, description = "Broadcast accepted (and landed, if wait_for_landing)"),
        (status = 400, description = "Malformed request body"),
        (status = 502, description = "Broadcast rejected or tx failed on-chain"),
    )
)]
#[tracing::instrument(skip(state, req), fields(
    title = %req.title,
    deposit = req.deposit_amount,
    gas_limit = req.gas_limit,
    fee_amount = req.fee_amount,
    wait_for_landing = req.wait_for_landing,
))]
pub async fn submit_store_code(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StoreCodeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!(
        wasm_b64_len = req.wasm_base64.len(),
        "POST /cosmos/ibc-wasm/store-code"
    );

    let wasm = BASE64.decode(req.wasm_base64.as_bytes()).map_err(|e| {
        tracing::error!(error = %e, "wasm_base64 decode failed");
        bad_request(format!("wasm_base64 not valid base64: {e}"))
    })?;
    tracing::debug!(wasm_bytes = wasm.len(), "store-code: decoded wasm");

    let result = state
        .cosmos
        .submit_store_code_proposal(
            wasm,
            req.title,
            req.summary,
            req.deposit_amount,
            req.gas_limit,
            req.fee_amount,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "submit_store_code_proposal failed");
            bad_gateway(e)
        })?;

    if result.code != 0 {
        tracing::warn!(
            tx_hash = %result.tx_hash,
            code = result.code,
            raw_log = %result.raw_log,
            "store-code broadcast rejected"
        );
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!(
                "broadcast rejected (code {}): {}",
                result.code, result.raw_log
            ),
        ));
    }

    tracing::info!(tx_hash = %result.tx_hash, "[cosmos] store-code (08-wasm) broadcast accepted");

    if !req.wait_for_landing {
        return Ok(Json(json!({
            "tx_hash": result.tx_hash,
            "code": result.code,
            "raw_log": result.raw_log,
        })));
    }

    let landed = state
        .cosmos
        .wait_for_tx(&result.tx_hash, Duration::from_secs(req.wait_timeout_secs))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, tx_hash = %result.tx_hash, "wait_for_tx failed");
            bad_gateway(e)
        })?;

    let landed_code = landed
        .pointer("/tx_response/code")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let landed_raw_log = landed
        .pointer("/tx_response/raw_log")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if landed_code != 0 {
        tracing::warn!(
            tx_hash = %result.tx_hash,
            landed_code,
            raw_log = %landed_raw_log,
            "store-code tx failed on-chain"
        );
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!(
                "tx failed on-chain (code {landed_code}, hash {}): {landed_raw_log}",
                result.tx_hash
            ),
        ));
    }

    let proposal_id = CosmosClient::extract_proposal_id(&landed);
    tracing::info!(
        tx_hash = %result.tx_hash,
        proposal_id = ?proposal_id,
        "[cosmos] store-code proposal landed"
    );

    Ok(Json(json!({
        "tx_hash": result.tx_hash,
        "code": result.code,
        "raw_log": result.raw_log,
        "tx_response": landed,
        "proposal_id": proposal_id,
    })))
}

/// Request body for [`submit_vote`].
#[derive(Deserialize, ToSchema)]
pub struct VoteRequest {
    /// Proposal id to vote on.
    pub proposal_id: u64,
    /// Vote option: `1` = YES, `2` = ABSTAIN, `3` = NO, `4` = NO_WITH_VETO.
    /// Defaults to YES (`1`).
    #[serde(default = "default_vote_option")]
    pub option: i32,
    /// Gas limit for the vote tx.
    pub gas_limit: u64,
    /// Tx fee in the chain's gas denom.
    pub fee_amount: u128,
}

fn default_vote_option() -> i32 {
    1
}

/// `POST /cosmos/gov/vote` — cast a vote on a gov proposal.
///
/// Body: [`VoteRequest`]. Signs with the funder key when configured (so the
/// vote carries the genesis validator's stake weight on a localnet), otherwise
/// falls back to the proposer key. A vote tx from an account with no bonded
/// stake will still be broadcast but contributes `0` weight to the tally.
#[utoipa::path(
    post,
    path = "/cosmos/gov/vote",
    tag = "Cosmos write",
    request_body = VoteRequest,
    responses(
        (status = 200, description = "Vote broadcast accepted"),
        (status = 502, description = "Vote rejected by the chain"),
    )
)]
#[tracing::instrument(skip(state, req), fields(
    proposal_id = req.proposal_id,
    option = req.option,
    gas_limit = req.gas_limit,
    fee_amount = req.fee_amount,
))]
pub async fn submit_vote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VoteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("POST /cosmos/gov/vote");

    let result = state
        .cosmos
        .submit_vote(req.proposal_id, req.option, req.gas_limit, req.fee_amount)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "submit_vote failed");
            bad_gateway(e)
        })?;

    if result.code != 0 {
        tracing::warn!(
            tx_hash = %result.tx_hash,
            code = result.code,
            raw_log = %result.raw_log,
            "vote rejected"
        );
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!("vote rejected (code {}): {}", result.code, result.raw_log),
        ));
    }

    tracing::info!(tx_hash = %result.tx_hash, "[cosmos] gov vote broadcast accepted");

    Ok(Json(json!({
        "tx_hash": result.tx_hash,
        "code": result.code,
        "raw_log": result.raw_log,
    })))
}

/// `GET /cosmos/proposer` — bech32 address derived from
/// `COSMOS_PROPOSER_PRIVATE_KEY`, or `null` when the key is unset.
#[utoipa::path(
    get,
    path = "/cosmos/proposer",
    tag = "Cosmos read",
    responses(
        (status = 200, description = "Proposer address (or null when unconfigured)"),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn proposer_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let address = state.cosmos.proposer_address();
    tracing::debug!(?address, "GET /cosmos/proposer");
    Json(json!({ "address": address }))
}

/// `GET /cosmos/funder` — bech32 address derived from
/// `COSMOS_FUNDER_PRIVATE_KEY`, or `null` when the key is unset.
#[utoipa::path(
    get,
    path = "/cosmos/funder",
    tag = "Cosmos read",
    responses(
        (status = 200, description = "Funder address (or null when unconfigured)"),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn funder_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let address = state.cosmos.funder_address();
    tracing::debug!(?address, "GET /cosmos/funder");
    Json(json!({ "address": address }))
}

/// Request body for [`submit_bank_send`].
#[derive(Deserialize, ToSchema)]
pub struct BankSendRequest {
    /// Destination bech32 address.
    pub to: String,
    /// Amount in the chain's gas denom.
    pub amount: u128,
    /// Gas limit for the tx.
    pub gas_limit: u64,
    /// Tx fee in the chain's gas denom.
    pub fee_amount: u128,
    /// When true, the handler first checks whether `to` already has an
    /// on-chain account and returns `{ skipped: true, ... }` without
    /// broadcasting. Useful for idempotent bootstrap funding. Defaults to false.
    #[serde(default)]
    pub skip_if_account_exists: bool,
}

/// `POST /cosmos/bank/send` — `cosmos.bank.v1beta1.MsgSend` signed by the
/// funder key.
///
/// Body: [`BankSendRequest`]. Always sends from the funder; the proposer can't
/// fund itself.
#[utoipa::path(
    post,
    path = "/cosmos/bank/send",
    tag = "Cosmos write",
    request_body = BankSendRequest,
    responses(
        (status = 200, description = "Bank send broadcast accepted (or skipped)"),
        (status = 502, description = "Bank send rejected by the chain"),
    )
)]
#[tracing::instrument(skip(state, req), fields(
    to = %req.to,
    amount = req.amount,
    gas_limit = req.gas_limit,
    fee_amount = req.fee_amount,
    skip_if_account_exists = req.skip_if_account_exists,
))]
pub async fn submit_bank_send(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BankSendRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("POST /cosmos/bank/send");

    if req.skip_if_account_exists {
        let exists = state.cosmos.account_exists(&req.to).await.map_err(|e| {
            tracing::error!(error = %e, to = %req.to, "account_exists check failed");
            bad_gateway(e)
        })?;
        if exists {
            tracing::debug!(to = %req.to, "bank-send skipped: account exists");
            return Ok(Json(json!({
                "skipped": true,
                "reason": "account already exists on chain",
                "to": req.to,
            })));
        }
    }

    let result = state
        .cosmos
        .submit_bank_send(req.to.clone(), req.amount, req.gas_limit, req.fee_amount)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, to = %req.to, amount = req.amount, "submit_bank_send failed");
            bad_gateway(e)
        })?;

    if result.code != 0 {
        tracing::warn!(
            tx_hash = %result.tx_hash,
            code = result.code,
            raw_log = %result.raw_log,
            "bank-send rejected"
        );
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!(
                "bank send rejected (code {}): {}",
                result.code, result.raw_log
            ),
        ));
    }

    tracing::info!(tx_hash = %result.tx_hash, to = %req.to, amount = req.amount, "[cosmos] bank-send broadcast accepted");

    Ok(Json(json!({
        "skipped": false,
        "to": req.to,
        "amount": req.amount.to_string(),
        "tx_hash": result.tx_hash,
        "code": result.code,
        "raw_log": result.raw_log,
    })))
}
