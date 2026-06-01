pub mod config;

use std::path::Path;

use anyhow::{bail, Result};

use crate::stellar::config::{StellarConfig, COMPOSE_SERVICE};
use crate::{logger, probe, run};

const WAIT_TIMEOUT_SECS: u64 = 300;

pub async fn start(cfg: &StellarConfig, root: &Path, http: &reqwest::Client) -> Result<()> {
    logger::banner(&format!("stellar start ({})", cfg.chain_id.as_str()));

    logger::detail(&format!("testnet — external endpoints ({})", cfg.rpc_url));

    if probe::http_ok(http, &cfg.status_url()).await {
        logger::ok("testnet reachable");
    } else {
        logger::warn("testnet not reachable — check the endpoints / your connection");
    }

    if probe::http_ok(http, &cfg.status_url()).await {
        logger::ok("already running");

        return Ok(());
    }

    logger::step(&format!("docker compose up -d {COMPOSE_SERVICE}"));
    run::compose(root, &["up", "-d", COMPOSE_SERVICE])?;

    if !probe::wait_http(http, &cfg.status_url(), WAIT_TIMEOUT_SECS).await {
        bail!("stellar not healthy within {WAIT_TIMEOUT_SECS}s (docker compose logs {COMPOSE_SERVICE})");
    }

    logger::ok("stellar running");

    Ok(())
}

pub fn stop(cfg: &StellarConfig, root: &Path) -> Result<()> {
    logger::banner("stellar stop");

    logger::detail("testnet — external, nothing to stop");

    logger::step(&format!("docker compose stop {COMPOSE_SERVICE}"));
    run::compose(root, &["stop", COMPOSE_SERVICE])?;

    logger::ok("stellar stopped");

    Ok(())
}

pub async fn status(cfg: &StellarConfig, http: &reqwest::Client) -> Result<()> {
    logger::banner(&format!("stellar status ({})", cfg.chain_id.as_str()));

    let up = probe::http_ok(http, &cfg.status_url()).await;
    logger::status_line(cfg.chain_id.as_str(), up, &cfg.rpc_url);

    let kind = "testnet (external)";

    logger::detail(&format!("network   {kind}"));
    logger::detail(&format!("gateway   {}", cfg.gateway_url));
    logger::detail(&format!("api       {}", cfg.api_url));

    Ok(())
}
