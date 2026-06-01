use std::path::Path;

use anyhow::Result;

use crate::{logger, run};

pub fn upload(root: &Path) -> Result<()> {
    logger::banner("contracts upload-wasm (light-client-wasm → Cosmos 08-wasm)");

    run::script(root, "upload-lc-wasm.sh", &[])
}
