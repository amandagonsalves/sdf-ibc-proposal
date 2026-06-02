//! Hermes config patching.
//!
//! The api bind-mounts the host `ci/hermes-config.toml` at the path given by
//! `HERMES_CONFIG_PATH` (default `/etc/hermes/config.toml`) so it can edit the
//! same file the hermes relayer reads.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::AppState;

fn err<E: std::fmt::Display>(status: StatusCode, e: E) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": e.to_string() })))
}

/// Request body for [`patch_wasm_checksum`].
#[derive(Deserialize, ToSchema)]
pub struct PatchChecksumRequest {
    /// 64-char lowercase hex sha256 of the wasm module to record in
    /// `wasm_checksum_hex`. Whitespace and case are normalized.
    pub checksum: String,
}

/// `POST /hermes/wasm-checksum` — replaces the first `wasm_checksum_hex = '…'`
/// line in the hermes config with the new checksum.
///
/// Body: [`PatchChecksumRequest`]. Returns `{ patched, path, checksum,
/// previous }` on success. Errors: `400` for malformed checksum, `404` if the
/// config file is missing, `422` if the `wasm_checksum_hex` line is absent.
///
/// Hermes does not re-read its config at runtime; restart the relayer after
/// patching for the change to take effect.
#[utoipa::path(
    post,
    path = "/hermes/wasm-checksum",
    tag = "Hermes",
    request_body = PatchChecksumRequest,
    responses(
        (status = 200, description = "Config patched"),
        (status = 400, description = "Checksum is not 64 lowercase hex chars"),
        (status = 404, description = "Hermes config file not found"),
        (status = 422, description = "wasm_checksum_hex line absent"),
        (status = 500, description = "Filesystem read/write error"),
    )
)]
#[tracing::instrument(skip(state, req))]
pub async fn patch_wasm_checksum(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PatchChecksumRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!("POST /hermes/wasm-checksum");

    let checksum = req.checksum.trim().to_ascii_lowercase();
    if checksum.len() != 64 || !checksum.chars().all(|c| c.is_ascii_hexdigit()) {
        tracing::warn!(
            checksum_len = checksum.len(),
            "rejected: malformed checksum"
        );
        return Err(err(
            StatusCode::BAD_REQUEST,
            "checksum must be 64 lowercase hex characters",
        ));
    }

    let path = state.hermes_config_path.as_str();
    if !Path::new(path).exists() {
        tracing::error!(%path, "hermes config not found");
        return Err(err(
            StatusCode::NOT_FOUND,
            format!("hermes config not found at {path}"),
        ));
    }

    let text = fs::read_to_string(path).map_err(|e| {
        tracing::error!(error = %e, %path, "read hermes config failed");
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read {path}: {e}"),
        )
    })?;

    let re = Regex::new(r"wasm_checksum_hex\s*=\s*'[^']*'").expect("static regex");

    let previous_match = re.find(&text);
    if previous_match.is_none() {
        tracing::warn!(%path, "wasm_checksum_hex line not found in hermes config");
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "wasm_checksum_hex line not found in hermes config",
        ));
    }
    let previous = previous_match.map(|m| m.as_str().to_string());

    let new_line = format!("wasm_checksum_hex = '{checksum}'");
    let new_text = re.replacen(&text, 1, new_line.as_str()).to_string();

    if new_text == text {
        tracing::info!(%path, %checksum, "hermes wasm_checksum_hex already set, no-op");
        return Ok(Json(json!({
            "patched": false,
            "path": path,
            "checksum": checksum,
            "previous": previous,
            "noop": true,
        })));
    }

    fs::write(path, new_text).map_err(|e| {
        tracing::error!(error = %e, %path, "write hermes config failed");
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("write {path}: {e}"),
        )
    })?;

    tracing::info!(%path, %checksum, previous = ?previous, "hermes wasm_checksum_hex patched");

    Ok(Json(json!({
        "patched": true,
        "path": path,
        "checksum": checksum,
        "previous": previous,
    })))
}
