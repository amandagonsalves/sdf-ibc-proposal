use std::path::Path;

use anyhow::Result;

use crate::{logger, run};

pub fn run(root: &Path, force: bool) -> Result<()> {
    logger::banner("contracts deploy-all (build + upload + deploy + wire router + write .env)");

    let env: Vec<(&str, &str)> = if force {
        vec![("FORCE_REDEPLOY", "1")]
    } else {
        vec![]
    };

    run::script(root, "upload-and-deploy-contracts.sh", &env)?;
    logger::hint("restart api+gateway to pick up the new IBC_CONTRACT_ID: stellaribc api restart --rebuild");

    Ok(())
}
