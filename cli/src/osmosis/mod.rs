pub mod config;

use std::path::Path;

use anyhow::{bail, Result};

use crate::osmosis::config::{OsmosisConfig, COMPOSE_SERVICE};
use crate::{logger, probe, run};

const WAIT_TIMEOUT_SECS: u64 = 300;

pub async fn start(cfg: &OsmosisConfig, root: &Path, http: &reqwest::Client) -> Result<()> {
    logger::banner(&format!("osmosis start ({})", cfg.chain_id.as_str()));

    if !cfg.is_local() {
        logger::detail(&format!("testnet — external endpoints ({})", cfg.rpc_url));

        if probe::http_ok(http, &cfg.status_url()).await {
            logger::ok("testnet reachable");
        } else {
            logger::warn("testnet not reachable — check the endpoints / your connection");
        }

        return Ok(());
    }

    if probe::http_ok(http, &cfg.status_url()).await {
        logger::ok("already running");

        return Ok(());
    }

    logger::step("docker compose up -d osmosis");
    run::compose(root, &["up", "-d", COMPOSE_SERVICE])?;

    if !probe::wait_http(http, &cfg.status_url(), WAIT_TIMEOUT_SECS).await {
        bail!("osmosis not healthy within {WAIT_TIMEOUT_SECS}s (docker compose logs osmosis)");
    }

    logger::ok("osmosis running");

    Ok(())
}

pub fn stop(cfg: &OsmosisConfig, root: &Path) -> Result<()> {
    logger::banner("osmosis stop");

    if !cfg.is_local() {
        logger::detail("testnet — external, nothing to stop");

        return Ok(());
    }

    logger::step("docker compose stop osmosis");
    run::compose(root, &["stop", COMPOSE_SERVICE])?;

    logger::ok("osmosis stopped");

    Ok(())
}

pub async fn status(cfg: &OsmosisConfig, http: &reqwest::Client) -> Result<()> {
    logger::banner(&format!("osmosis status ({})", cfg.chain_id.as_str()));

    let up = probe::http_ok(http, &cfg.status_url()).await;
    logger::status_line(cfg.chain_id.as_str(), up, &cfg.rpc_url);

    let kind = if cfg.is_local() {
        "local (docker compose)"
    } else {
        "testnet (external)"
    };
    logger::detail(&format!("network   {kind}"));
    logger::detail(&format!("rest      {}", cfg.rest_url));
    logger::detail(&format!("grpc      {}", cfg.grpc_url));

    Ok(())
}
