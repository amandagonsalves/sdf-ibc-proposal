use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::Config;
use crate::{probe, run, ui};

pub fn install(root: &Path) -> Result<()> {
    ui::banner("install — cargo install stellaribc");
    let crate_dir = root.join("crates/cli");
    ui::step("cargo install --path crates/cli --force");
    run::command(
        root,
        "cargo",
        &[
            "install",
            "--path",
            crate_dir.to_str().unwrap_or("crates/cli"),
            "--force",
        ],
    )?;

    let bin_dir = cargo_bin_dir();
    ui::ok(&format!(
        "installed: {}",
        bin_dir.join("stellaribc").display()
    ));
    if on_path(&bin_dir.as_path()) {
        ui::ok(&format!(
            "{} is on PATH — run: stellaribc status",
            bin_dir.display()
        ));
    } else {
        ui::warn(&format!("{} is not on PATH", bin_dir.display()));
        ui::detail("add it to your shell profile (bash/zsh):");
        ui::detail(&format!("export PATH=\"{}:$PATH\"", bin_dir.display()));
    }
    Ok(())
}

fn cargo_bin_dir() -> PathBuf {
    if let Ok(home) = std::env::var("CARGO_HOME") {
        return PathBuf::from(home).join("bin");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cargo").join("bin");
    }
    PathBuf::from(".cargo/bin")
}

fn on_path(dir: &Path) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|entry| entry == dir))
        .unwrap_or(false)
}

pub async fn doctor(root: &Path, cfg: &Config, http: &reqwest::Client) -> Result<()> {
    ui::banner("doctor — prerequisites & configuration");

    ui::step("Toolchain");
    check(
        "docker",
        run::has("docker"),
        "required to run the chain + services",
    );
    check(
        "stellar",
        run::has("stellar"),
        "Soroban CLI, used to build/deploy contracts",
    );
    check(
        "cargo",
        run::has("cargo"),
        "builds the wasm light client + this CLI",
    );

    ui::step("Repository");
    ui::ok(&format!("repo root: {}", root.display()));
    let env_file = root.join(".env");
    if env_file.exists() {
        ui::ok(".env present");
    } else {
        ui::fail(".env missing — copy .env.example and fill it in");
    }

    ui::step("Configuration");
    flag(
        "STELLAR_SIGNING_KEY",
        !cfg.stellar_signing_key.is_empty(),
        "needed to deploy + sign on Stellar",
    );
    flag(
        "IBC_CONTRACT_ID",
        !cfg.ibc_contract_id.is_empty(),
        "router address (set by `stellaribc deploy-contracts`)",
    );
    flag(
        "TRANSFER_CONTRACT_ID",
        !cfg.transfer_contract_id.is_empty(),
        "transfer-app address",
    );
    flag(
        "STELLAR_CLIENT_ID",
        !cfg.stellar_client_id.is_empty(),
        "08-wasm client id (set by `stellaribc client create-stellar`)",
    );

    ui::step("Services");
    let cosmos = probe::http_ok(http, &cfg.cosmos_node_info_url()).await;
    ui::status_line(&cfg.cosmos_chain_id, cosmos, &cfg.cosmos_rest_url);
    let api = probe::http_ok(http, &cfg.api_health_url()).await;
    ui::status_line("stellar-api", api, &cfg.api_url);
    let gw = probe::tcp_ok(&cfg.gateway_grpc_addr);
    ui::status_line("gateway-grpc", gw, &cfg.gateway_grpc_addr);

    if !cosmos || !api {
        ui::hint("bring the stack up with: stellaribc up");
    }
    Ok(())
}

pub async fn status(_root: &Path, cfg: &Config, http: &reqwest::Client) -> Result<()> {
    ui::banner("status");

    ui::step("Chains & services");
    let cosmos = probe::http_ok(http, &cfg.cosmos_node_info_url()).await;
    ui::status_line(&cfg.cosmos_chain_id, cosmos, &cfg.cosmos_rest_url);
    let api = probe::http_ok(http, &cfg.api_health_url()).await;
    ui::status_line("stellar-api", api, &cfg.api_url);
    let gw = probe::tcp_ok(&cfg.gateway_grpc_addr);
    ui::status_line("gateway-grpc", gw, &cfg.gateway_grpc_addr);

    ui::step("Endpoints");
    ui::detail(&format!("cosmos rpc   {}", cfg.cosmos_rpc_url));
    ui::detail(&format!("hermes cfg   {}", cfg.hermes_config));

    ui::step("Stellar contracts (from .env)");
    contract("router", &cfg.ibc_contract_id);
    contract("transfer-app", &cfg.transfer_contract_id);
    contract("deployer", &cfg.deployer_address);

    ui::step("Created clients");
    if !api {
        ui::warn("api unreachable — start it with `stellaribc up` to list clients");
    } else if let Some(value) = probe::get_json(http, &cfg.clients_url()).await {
        print_clients(&value);
    } else {
        ui::warn("could not read /stellar/clients");
    }
    Ok(())
}

pub fn up(root: &Path, cosmos_only: bool, stellar_only: bool) -> Result<()> {
    ui::banner("up — docker compose");
    let services: Vec<&str> = if cosmos_only {
        vec!["osmosis"]
    } else if stellar_only {
        vec!["api", "gateway"]
    } else {
        vec!["osmosis", "api", "gateway"]
    };
    ui::step(&format!("starting: {}", services.join(", ")));
    let mut args = vec!["up", "-d"];
    args.extend_from_slice(&services);
    run::compose(root, &args)?;
    ui::ok("services started (detached)");
    ui::hint("check readiness with: stellaribc status");
    Ok(())
}

pub fn down(root: &Path, volumes: bool) -> Result<()> {
    ui::banner("down — docker compose");
    let mut args = vec!["down"];
    if volumes {
        args.push("--volumes");
    }
    run::compose(root, &args)?;
    ui::ok("stack stopped");
    Ok(())
}

pub fn bootstrap(
    root: &Path,
    skip_images: bool,
    skip_contracts: bool,
    skip_wasm: bool,
    skip_keys: bool,
    force_redeploy: bool,
) -> Result<()> {
    ui::banner("bootstrap (F0)");
    let mut env: Vec<(&str, &str)> = Vec::new();
    if skip_images {
        env.push(("SKIP_IMAGE_BUILD", "1"));
    }
    if skip_contracts {
        env.push(("SKIP_CONTRACT_DEPLOY", "1"));
    }
    if skip_wasm {
        env.push(("SKIP_LC_WASM_UPLOAD", "1"));
    }
    if skip_keys {
        env.push(("SKIP_HERMES_KEYS", "1"));
    }
    if force_redeploy {
        env.push(("FORCE_REDEPLOY", "1"));
    }
    run::script(root, "f0-bootstrap.sh", &env)?;
    ui::hint("next: stellaribc client create-cosmos   (F1.1)");
    Ok(())
}

pub fn build_images(root: &Path, target: &str) -> Result<()> {
    ui::banner("build-images");
    let scripts: &[&str] = match target {
        "api" => &["build-api-image.sh"],
        "gateway" => &["build-gateway-image.sh"],
        "hermes" => &["build-hermes-image.sh"],
        _ => &[
            "build-api-image.sh",
            "build-gateway-image.sh",
            "build-hermes-image.sh",
        ],
    };
    for s in scripts {
        ui::step(s);
        run::script(root, s, &[])?;
    }
    Ok(())
}

pub fn deploy_contracts(root: &Path, force: bool) -> Result<()> {
    ui::banner("deploy-contracts");
    let env: Vec<(&str, &str)> = if force {
        vec![("FORCE_REDEPLOY", "1")]
    } else {
        vec![]
    };
    run::script(root, "upload-and-deploy-contracts.sh", &env)?;
    ui::hint(
        "restart api+gateway so they pick up the new IBC_CONTRACT_ID: stellaribc up --stellar",
    );
    Ok(())
}

pub fn upload_wasm(root: &Path) -> Result<()> {
    ui::banner("upload-wasm (light-client-wasm)");
    run::script(root, "upload-lc-wasm.sh", &[])
}

pub fn import_keys(root: &Path) -> Result<()> {
    ui::banner("import-keys (hermes relayer)");
    run::script(root, "hermes-keys.sh", &[])
}

pub fn client_create_cosmos(root: &Path, cli_path: bool) -> Result<()> {
    ui::banner("client create-cosmos (F1.1 — Cosmos client on Stellar)");
    let script = if cli_path {
        "f1-create-client.sh"
    } else {
        "f1-create-cosmos-client.sh"
    };
    run::script(root, script, &[])?;
    ui::hint("next: stellaribc client create-stellar   (F1.2)");
    Ok(())
}

pub fn client_create_stellar(root: &Path, force: bool) -> Result<()> {
    ui::banner("client create-stellar (F1.2 — Stellar client on Cosmos, 08-wasm)");
    let env: Vec<(&str, &str)> = if force {
        vec![("FORCE_RECREATE", "1")]
    } else {
        vec![]
    };
    run::script(root, "f1-create-stellar-client.sh", &env)?;
    ui::hint("next: register counterparties — stellaribc counterparty stellar / stellaribc counterparty cosmos");
    Ok(())
}

pub async fn client_list(cfg: &Config, http: &reqwest::Client) -> Result<()> {
    ui::banner("client list");
    if !probe::http_ok(http, &cfg.api_health_url()).await {
        ui::warn("api unreachable — start it with `stellaribc up`");
        return Ok(());
    }
    match probe::get_json(http, &cfg.clients_url()).await {
        Some(value) => print_clients(&value),
        None => ui::warn("could not read /stellar/clients"),
    }
    Ok(())
}

pub fn counterparty(root: &Path, side: &str) -> Result<()> {
    let (label, script) = match side {
        "stellar" => (
            "counterparty stellar (F1.3 — register Cosmos client as counterparty on Stellar)",
            "f1-register-counterparty-stellar.sh",
        ),
        _ => (
            "counterparty cosmos (F1.4 — register Stellar client as counterparty on Cosmos)",
            "f1-register-counterparty-cosmos.sh",
        ),
    };
    ui::banner(label);

    if run::script_exists(root, script) {
        return run::script(root, script, &[]);
    }

    ui::warn(&format!("{script} not present yet."));
    ui::detail("Counterparty registration is the current frontier (TASKS.md, Task 3).");
    ui::detail("It is blocked on migrating the gateway's register_counterparty RPC to the");
    ui::detail("prepare->sign->submit flow (only create_client is wired today). Once the");
    ui::detail(&format!(
        "script lands at ci/flows/{script}, this command runs it automatically."
    ));
    Ok(())
}

fn print_clients(value: &serde_json::Value) {
    let Some(clients) = value.get("clients").and_then(|c| c.as_array()) else {
        ui::warn("unexpected response shape from /stellar/clients");
        return;
    };
    if clients.is_empty() {
        ui::detail("no clients created yet");
        return;
    }
    for client in clients {
        let client_type = client
            .get("client_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let ids = client
            .get("client_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        ui::ok(&format!("{client_type}: {ids}"));
    }
}

fn check(name: &str, present: bool, note: &str) {
    if present {
        ui::ok(&format!("{name} found"));
    } else {
        ui::fail(&format!("{name} not found — {note}"));
    }
}

fn flag(name: &str, set: bool, note: &str) {
    if set {
        ui::ok(&format!("{name} set"));
    } else {
        ui::warn(&format!("{name} unset — {note}"));
    }
}

fn contract(label: &str, id: &str) {
    if id.is_empty() {
        ui::warn(&format!("{label:<13} (unset)"));
    } else {
        ui::ok(&format!("{label:<13} {id}"));
    }
}
