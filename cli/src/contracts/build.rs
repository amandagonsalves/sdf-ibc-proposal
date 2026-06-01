use std::path::Path;

use anyhow::Result;

use crate::{logger, run};

pub fn run(root: &Path) -> Result<()> {
    logger::banner("contracts build");

    let contracts_dir = root.join("contracts");

    logger::step("stellar contract build --profile contract");
    run::command(
        &contracts_dir,
        "stellar",
        &["contract", "build", "--profile", "contract"],
    )?;

    logger::ok("built → contracts/target/wasm32v1-none/contract/");

    Ok(())
}
