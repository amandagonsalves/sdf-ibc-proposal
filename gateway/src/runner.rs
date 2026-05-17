use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use soroban_client::account::AccountBehavior;
use soroban_client::keypair::{Keypair, KeypairBehavior};

use crate::{
    api::account::{account, balance, get_account},
    config::GatewayConfig,
    state::AppState,
};

pub async fn run(cfg: GatewayConfig) {
    let _soroban = stellar_gateway_store::soroban::SorobanClient::new(&cfg.rpc_url);

    let http_addr = cfg.http_addr();
    let app_state = Arc::new(AppState::new());

    tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(http_health))
            .route("/account/{address}", get(account))
            .route("/balance/{address}", get(balance))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();

        tracing::info!("HTTP server listening on {}", http_addr);

        axum::serve(listener, app).await.unwrap();
    });

    let grpc_addr = cfg.grpc_addr();

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("gRPC reflection service failed to build");

    tracing::info!("gRPC server listening on {}", grpc_addr);

    tonic::transport::Server::builder()
        .add_service(reflection_service)
        .add_service(health_service)
        // .add_service(ClientServiceServer::new(client_handler))
        // .add_service(PacketServiceServer::new(packet_handler))
        // .add_service(QueryServiceServer::new(query_handler))
        // .add_service(CounterpartyServiceServer::new(counterparty_handler))
        .serve(grpc_addr)
        .await
        .expect("gRPC server failed");
}

async fn http_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let keypair =
        Keypair::from_secret(&state.signing_key).expect("could not get keypair from secret key");
    let public_key = keypair.public_key().to_string();
    let account = get_account(&state, &public_key).await;
    (
        StatusCode::OK,
        format!(
            "Stellar Gateway is up and the signer {} is ready.",
            account.account_id()
        ),
    )
}
