use std::path::Path;

use anyhow::Result;

use super::{last_line, net_flags};
use crate::config::Config;
use crate::{logger, run};

pub fn run(cfg: &Config, root: &Path, wasm: &str, ctor: &[String]) -> Result<()> {
    logger::banner("contracts deploy");

    let mut args: Vec<String> = vec![
        "contract".into(),
        "deploy".into(),
        "--source".into(),
        cfg.deployer_identity.clone(),
    ];
    args.extend(net_flags(cfg));
    args.push("--wasm".into());
    args.push(wasm.to_string());

    if !ctor.is_empty() {
        args.push("--".into());
        args.extend(ctor.iter().cloned());
    }

    logger::step(&format!("stellar contract deploy --wasm {wasm}"));

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let id = last_line(&run::capture(root, "stellar", &refs)?);

    logger::ok(&format!("contract id: {id}"));
    println!("{id}");

    Ok(())
}
