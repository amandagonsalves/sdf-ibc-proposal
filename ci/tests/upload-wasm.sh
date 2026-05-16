#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)
WASM_FILE="${REPO_ROOT}/target/wasm32-unknown-unknown/release/stellar_ibc_light_client.wasm"
HERMES_CONFIG="${HOME}/.hermes/config.toml"
CHAIN_ID="cardano-entrypoint"

echo "=== 08-wasm upload test ==="

if [[ ! -f "${WASM_FILE}" ]]; then
  echo "ERROR: WASM not found at ${WASM_FILE}"
  echo "  Run: bash ci/entrypoint.sh"
  exit 1
fi

echo "WASM: ${WASM_FILE} ($(wc -c < "${WASM_FILE}") bytes)"

echo ""
echo "Step 1: Checking ${CHAIN_ID} is reachable..."
if ! hermes --config "${HERMES_CONFIG}" health-check 2>/dev/null; then
  echo "SKIP: ${CHAIN_ID} is not reachable."
  echo "  Start it with: cd cardano-ibc-incubator/cosmos/cardano-entrypoint && ignite chain serve -y"
  exit 0
fi

echo ""
echo "Step 2: Uploading WASM to ${CHAIN_ID}..."
UPLOAD_OUTPUT=$(hermes --config "${HERMES_CONFIG}" \
  client store wasm-code \
  --chain "${CHAIN_ID}" \
  --wasm-file "${WASM_FILE}" 2>&1) || {
  echo "ERROR: hermes client store wasm-code failed:"
  echo "${UPLOAD_OUTPUT}"
  exit 1
}

echo "${UPLOAD_OUTPUT}"

CHECKSUM=$(echo "${UPLOAD_OUTPUT}" | grep -oE '[0-9a-f]{64}' | head -1 || true)

if [[ -z "${CHECKSUM}" ]]; then
  echo "ERROR: Could not extract checksum from upload output."
  exit 1
fi

echo ""
echo "Checksum: ${CHECKSUM}"

echo ""
echo "Step 3: Verifying checksum is registered on-chain..."
CHECKSUMS=$(hermes --config "${HERMES_CONFIG}" \
  query wasm checksums \
  --chain "${CHAIN_ID}" 2>&1)

echo "${CHECKSUMS}"

if echo "${CHECKSUMS}" | grep -q "${CHECKSUM}"; then
  echo ""
  echo "SUCCESS: Stellar light client WASM registered on ${CHAIN_ID}"
  echo "  Checksum: ${CHECKSUM}"
  echo "  The chain can now create 10-stellar clients via 08-wasm."
else
  echo "ERROR: Checksum ${CHECKSUM} not found in on-chain list."
  exit 1
fi
