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
        .route("/account/{address}", get(services::account::account))
        .route("/balance/{address}", get(services::balance::balance))
        .route("/tx/xdr", get(services::tx::get_unsigned_tx))
        .route("/tx/{tx_hash}", get(services::tx::get_signed_tx))
        .route("/tx/sign", post(services::tx::sign_tx))
        .route("/tx/submit", post(services::tx::submit_signed_tx))
        .with_state(state)
}

async fn health() -> &'static str {
    "Stellar IBC API is healthy."
}

pub async fn serve(addr: SocketAddr, state: Arc<AppState>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("HTTP server listening on {}", addr);

    axum::serve(listener, router(state)).await?;

    Ok(())
}

pub async fn run(cfg: ApiConfig) -> anyhow::Result<()> {
    let rpc = RpcClient::new(cfg.rpc_url.as_str()).expect("could not create a new rpc client");

    let state = Arc::new(AppState::new(rpc, cfg.signing_key.clone()));

    serve(cfg.addr(), state).await
}
