use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::repo;

pub fn script(root: &Path, name: &str, env: &[(&str, &str)]) -> Result<()> {
    let path = repo::script_path(root, name);
    if !path.exists() {
        bail!("flow script not found: {}", path.display());
    }

    let mut cmd = Command::new("bash");
    cmd.arg(&path).current_dir(root);
    for (key, value) in env {
        cmd.env(key, value);
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to spawn {}", path.display()))?;
    if !status.success() {
        bail!("{name} exited with {status}");
    }
    Ok(())
}

pub fn script_exists(root: &Path, name: &str) -> bool {
    repo::script_path(root, name).exists()
}

pub fn command(root: &Path, program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .with_context(|| format!("failed to spawn {program}"))?;
    if !status.success() {
        bail!("{program} {} exited with {status}", args.join(" "));
    }
    Ok(())
}

pub fn compose(root: &Path, extra: &[&str]) -> Result<()> {
    let mut args = vec!["compose", "--profile", "local", "--profile", "hermes"];
    args.extend_from_slice(extra);
    command(root, "docker", &args)
}

pub fn has(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
