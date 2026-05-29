use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use stellar_ibc_core::rpc::RpcClient;
use tokio::net::TcpListener;

use crate::{config::ApiConfig, services, AppState};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ledger/latest", get(services::ledgers::latest))
        .route("/ledger/{sequence}", get(services::ledgers::get_one))
        .route("/events", get(services::events::list))
        .route("/account/{address}", get(services::account::account))
        .route("/balance/{address}", get(services::balance::balance))
        .route("/tx/xdr", get(services::tx::get_unsigned_tx))
        .route("/tx/{tx_hash}", get(services::tx::get_signed_tx))
        .route("/tx/sign", post(services::tx::sign_tx))
        .route("/tx/submit", post(services::tx::submit_signed_tx))
        .with_state(state)
}

async fn health() -> &'static str {
    tracing::debug!("GET /health");
    "Stellar IBC API is healthy."
}

pub async fn serve(addr: SocketAddr, state: Arc<AppState>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "stellar-api HTTP server listening");

    axum::serve(listener, router(state)).await?;

    Ok(())
}

pub async fn run(cfg: ApiConfig) -> anyhow::Result<()> {
    tracing::info!(
        host = %cfg.host,
        port = cfg.port,
        rpc_url = %cfg.rpc_url,
        signing_key_configured = !cfg.signing_key.is_empty(),
        "starting stellar-api"
    );

    let rpc = RpcClient::new(cfg.rpc_url.as_str()).expect("could not create a new rpc client");

    let state = Arc::new(AppState::new(rpc, cfg.signing_key.clone()));

    serve(cfg.addr(), state).await
}
