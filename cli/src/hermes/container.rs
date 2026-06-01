use std::path::Path;

use anyhow::Result;

use crate::hermes::config::HermesConfig;
use crate::{logger, run};

const SERVICE: &str = "hermes";

pub fn start(cfg: &HermesConfig, root: &Path, pull: bool) -> Result<()> {
    logger::banner("hermes start (relayer container)");
    logger::detail(&format!("image: {}", cfg.image.reference()));

    if pull {
        run::compose(root, &["pull", SERVICE])?;
    }

    logger::step("docker compose up -d hermes");
    run::compose(root, &["up", "-d", SERVICE])?;

    logger::ok("hermes started");

    Ok(())
}

pub fn stop(root: &Path) -> Result<()> {
    logger::banner("hermes stop");

    logger::step("docker compose stop hermes");
    run::compose(root, &["stop", SERVICE])?;

    logger::ok("hermes stopped");

    Ok(())
}

pub fn restart(cfg: &HermesConfig, root: &Path, pull: bool) -> Result<()> {
    logger::banner("hermes restart");
    logger::detail(&format!("image: {}", cfg.image.reference()));

    if pull {
        run::compose(root, &["pull", SERVICE])?;

        logger::step("docker compose up -d --force-recreate hermes");
        run::compose(root, &["up", "-d", "--force-recreate", SERVICE])?;
    } else {
        logger::step("docker compose restart hermes");
        run::compose(root, &["restart", SERVICE])?;
    }

    logger::ok("hermes restarted");

    Ok(())
}
