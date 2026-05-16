#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
WASM_FILE="${REPO_ROOT}/target/wasm32-unknown-unknown/release/stellar_ibc_light_client.wasm"
HERMES_CONFIG="${HOME}/.hermes/config.toml"
TESTS_DIR="${SCRIPT_DIR}/tests"
MNEMONIC_FILE="${SCRIPT_DIR}/relayer-mnemonic.txt"

# ── Prerequisites ─────────────────────────────────────────────────────────────

if ! command -v hermes &>/dev/null; then
  echo "ERROR: hermes not found. Install a pre-built binary:"
  echo ""
  echo "  macOS ARM:"
  echo "    curl -L https://github.com/informalsystems/hermes/releases/download/v1.13.2/hermes-v1.13.2-aarch64-apple-darwin.tar.gz \\"
  echo "      | tar -xz && mkdir -p ~/.local/bin && mv hermes ~/.local/bin/"
  echo ""
  echo "  Linux x86:"
  echo "    curl -L https://github.com/informalsystems/hermes/releases/download/v1.13.2/hermes-v1.13.2-x86_64-unknown-linux-gnu.tar.gz \\"
  echo "      | tar -xz && mkdir -p ~/.local/bin && mv hermes ~/.local/bin/"
  echo ""
  echo "  Then add ~/.local/bin to your PATH."
  exit 1
fi

if [[ ! -f "${REPO_ROOT}/Cargo.toml" ]]; then
  echo "ERROR: Run this script from inside the stellar-ibc repository."
  exit 1
fi

# ── Build WASM ────────────────────────────────────────────────────────────────

echo "==> Building stellar-ibc-light-client (wasm32)..."
cd "${REPO_ROOT}"
cargo build \
  --target wasm32-unknown-unknown \
  --no-default-features \
  -p stellar-ibc-light-client \
  --release

echo "    ${WASM_FILE}"
echo "    $(wc -c < "${WASM_FILE}") bytes"

# ── Configure Hermes ──────────────────────────────────────────────────────────

echo ""
echo "==> Configuring Hermes..."
mkdir -p "${HOME}/.hermes"
cp "${SCRIPT_DIR}/hermes-config.toml" "${HERMES_CONFIG}"
echo "    Config written to ${HERMES_CONFIG}"

# ── Add relayer key ───────────────────────────────────────────────────────────

echo ""
echo "==> Adding relayer key to Hermes..."

if [[ -f "${MNEMONIC_FILE}" ]]; then
  hermes --config "${HERMES_CONFIG}" \
    keys add \
    --chain cardano-entrypoint \
    --mnemonic-file "${MNEMONIC_FILE}" \
    --overwrite
  echo "    Key added from ${MNEMONIC_FILE}"
elif [[ -n "${RELAYER_MNEMONIC:-}" ]]; then
  TMP_MNEMONIC=$(mktemp)
  echo "${RELAYER_MNEMONIC}" > "${TMP_MNEMONIC}"
  hermes --config "${HERMES_CONFIG}" \
    keys add \
    --chain cardano-entrypoint \
    --mnemonic-file "${TMP_MNEMONIC}" \
    --overwrite
  rm "${TMP_MNEMONIC}"
  echo "    Key added from RELAYER_MNEMONIC env var"
else
  echo "    WARNING: No mnemonic found — skipping key import."
  echo "    Provide either:"
  echo "      ci/relayer-mnemonic.txt  (copy from incubator config.yml)"
  echo "      RELAYER_MNEMONIC env var"
fi

# ── Run tests ─────────────────────────────────────────────────────────────────

echo ""
echo "==> Running tests from ${TESTS_DIR}..."
PASS=0
SKIP=0
FAIL=0

for t in "${TESTS_DIR}"/*.sh; do
  echo ""
  echo "--- $(basename "$t")"
  set +e
  bash "$t"
  STATUS=$?
  set -e
  if [[ $STATUS -eq 0 ]]; then
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
    echo "FAILED: $t (exit $STATUS)"
  fi
done

echo ""
echo "==> Results: ${PASS} passed, ${SKIP} skipped, ${FAIL} failed"
[[ $FAIL -eq 0 ]] || exit 1
