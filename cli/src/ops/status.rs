use anyhow::Result;

use crate::config::Config;
use crate::{logger, probe, shared};

pub async fn run(cfg: &Config, http: &reqwest::Client) -> Result<()> {
    logger::banner("status");

    logger::step("Chains & services");

    let cosmos = probe::http_ok(http, &cfg.cosmos_node_info_url()).await;
    logger::status_line(&cfg.cosmos_chain_id, cosmos, &cfg.cosmos_rest_url);

    let api = probe::http_ok(http, &cfg.api_health_url()).await;
    logger::status_line("stellar-api", api, &cfg.api_url);

    let gateway = probe::tcp_ok(&cfg.gateway_grpc_addr);
    logger::status_line("gateway-grpc", gateway, &cfg.gateway_grpc_addr);

    logger::step("Endpoints");
    logger::detail(&format!("cosmos rpc   {}", cfg.cosmos_rpc_url));
    logger::detail(&format!("hermes cfg   {}", cfg.hermes_config));

    logger::step("Stellar contracts (from .env)");
    shared::contract("router", &cfg.ibc_contract_id);
    shared::contract("transfer-app", &cfg.transfer_contract_id);
    shared::contract("deployer", &cfg.deployer_address);

    logger::step("Created clients");

    if !api {
        logger::warn("api unreachable — start it with `stellaribc up` to list clients");

        return Ok(());
    }

    match probe::get_json(http, &cfg.clients_url()).await {
        Some(value) => shared::print_clients(&value),
        None => logger::warn("could not read /stellar/clients"),
    }

    Ok(())
}
