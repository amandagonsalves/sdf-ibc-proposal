pub mod build;
pub mod deploy;
pub mod deploy_all;
pub mod invoke;
pub mod upload;
pub mod wasm;

use crate::config::Config;

pub(crate) fn net_flags(cfg: &Config) -> Vec<String> {
    vec![
        "--rpc-url".to_string(),
        cfg.stellar_rpc_url.clone(),
        "--network-passphrase".to_string(),
        cfg.network_passphrase.clone(),
    ]
}

pub(crate) fn last_line(out: &str) -> String {
    out.lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}
