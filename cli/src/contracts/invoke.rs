use std::path::Path;

use anyhow::{bail, Result};

use super::net_flags;
use crate::config::Config;
use crate::{logger, run};

pub fn run(cfg: &Config, root: &Path, id: &str, call: &[String]) -> Result<()> {
    logger::banner("contracts invoke");

    if call.is_empty() {
        bail!("nothing to invoke — pass the function + args after `--`, e.g. `-- register_port --port_id transfer --app_address C...`");
    }

    let mut args: Vec<String> = vec![
        "contract".into(),
        "invoke".into(),
        "--source".into(),
        cfg.deployer_identity.clone(),
    ];
    args.extend(net_flags(cfg));
    args.push("--id".into());
    args.push(id.to_string());
    args.push("--".into());
    args.extend(call.iter().cloned());

    logger::step(&format!("stellar contract invoke --id {id} -- {}", call.join(" ")));

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run::command(root, "stellar", &refs)?;

    Ok(())
}
