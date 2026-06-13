use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use anyhow::{bail, Context, Result};

use crate::logger;

pub fn command(root: &Path, program: &str, args: &[&str]) -> Result<()> {
    // When a spinner is running, stream the child's output into it (caribic
    // style) so its latest line shows as the spinner message instead of
    // clobbering the in-place repaint. Otherwise inherit stdio.
    match logger::current_bar() {
        Some(bar) => command_streaming(root, program, args, &bar),
        None => command_inherit(root, program, args),
    }
}

fn command_inherit(root: &Path, program: &str, args: &[&str]) -> Result<()> {
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

fn command_streaming(
    root: &Path,
    program: &str,
    args: &[&str],
    bar: &indicatif::ProgressBar,
) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn {program}"))?;

    let stdout = child.stdout.take().context("child stdout unavailable")?;
    let stderr = child.stderr.take().context("child stderr unavailable")?;

    let (tx, rx) = mpsc::channel::<String>();
    let tx_err = tx.clone();

    let out_handle = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = tx.send(line);
        }
    });
    let err_handle = thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            let _ = tx_err.send(line);
        }
    });

    // Drains until both reader threads drop their senders (child pipes closed).
    for line in rx {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            bar.set_message(trimmed.to_string());
        }
    }

    let _ = out_handle.join();
    let _ = err_handle.join();

    let status = child.wait().with_context(|| format!("{program} failed"))?;
    if !status.success() {
        bail!("{program} {} exited with {status}", args.join(" "));
    }
    Ok(())
}

pub fn piped(root: &Path, program: &str, args: &[&str], input: &str) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn {program}"))?;

    child
        .stdin
        .as_mut()
        .context("child stdin unavailable")?
        .write_all(input.as_bytes())
        .context("failed to write to child stdin")?;

    let status = child.wait().with_context(|| format!("{program} failed"))?;

    if !status.success() {
        bail!("{program} exited with {status}");
    }

    Ok(())
}

pub fn capture(root: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| format!("failed to spawn {program}"))?;

    if !output.status.success() {
        bail!("{program} exited with {}", output.status);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn capture_quiet(root: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .stderr(Stdio::null())
        .output()
        .with_context(|| format!("failed to spawn {program}"))?;

    if !output.status.success() {
        bail!("{program} exited with {}", output.status);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn capture_all(root: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to spawn {program}"))?;

    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        bail!("{program} exited with {}:\n{combined}", output.status);
    }

    Ok(combined)
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
