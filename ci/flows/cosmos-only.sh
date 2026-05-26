#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
CI_DIR=$(cd "${SCRIPT_DIR}/.." && pwd)
REPO_ROOT=$(cd "${CI_DIR}/.." && pwd)

source "${SCRIPT_DIR}/_env.sh"
load_env_file "${REPO_ROOT}/.env"

INCUBATOR_REPO="${INCUBATOR_REPO:-${REPO_ROOT}/../cardano-ibc-incubator}"
COSMOS_CHAIN_NAME="${COSMOS_CHAIN_NAME:-osmosis}"
COSMOS_NETWORK="${COSMOS_NETWORK:-local}"
COSMOS_RPC_URL="${COSMOS_RPC_URL:-http://127.0.0.1:26658}"
COSMOS_REST_URL="${COSMOS_REST_URL:-http://127.0.0.1:1318}"
WAIT_TIMEOUT_SEC="${WAIT_TIMEOUT_SEC:-180}"

echo "=== Cosmos-only bootstrap (workaround for caribic gateway-app failure) ==="
echo "  chain          : ${COSMOS_CHAIN_NAME}"
echo "  network        : ${COSMOS_NETWORK}"
echo "  incubator repo : ${INCUBATOR_REPO}"
echo ""

if [[ ! -d "${INCUBATOR_REPO}" ]]; then
  echo "ERROR: INCUBATOR_REPO not found: ${INCUBATOR_REPO}"
  echo "Set INCUBATOR_REPO in .env or clone https://github.com/cardano-foundation/cardano-ibc-incubator next to stellar-ibc."
  exit 1
fi

if ! command -v caribic > /dev/null 2>&1; then
  echo "ERROR: caribic binary not on PATH."
  echo "Build + install from ${INCUBATOR_REPO}/caribic (cargo install --path .)"
  exit 1
fi

echo "Starting ${COSMOS_CHAIN_NAME} (${COSMOS_NETWORK}) via 'caribic chain start' ..."
( cd "${INCUBATOR_REPO}" && caribic chain start --chain "${COSMOS_CHAIN_NAME}" --network "${COSMOS_NETWORK}" )

echo ""
echo "Waiting for ${COSMOS_RPC_URL}/status to report block height > 0 (timeout ${WAIT_TIMEOUT_SEC}s) ..."
deadline=$(( $(date +%s) + WAIT_TIMEOUT_SEC ))
while (( $(date +%s) < deadline )); do
  body=$(curl -sf "${COSMOS_RPC_URL}/status" 2>/dev/null || true)
  if [[ -n "${body}" ]]; then
    height=$(echo "${body}" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d.get("result",{}).get("sync_info",{}).get("latest_block_height","0"))' 2>/dev/null || echo 0)
    if [[ "${height}" =~ ^[0-9]+$ ]] && (( height > 0 )); then
      echo "  Reachable. height=${height}"
      break
    fi
  fi
  sleep 3
done

if ! curl -sf "${COSMOS_RPC_URL}/status" > /dev/null 2>&1; then
  echo "ERROR: ${COSMOS_CHAIN_NAME} did not become healthy within ${WAIT_TIMEOUT_SEC}s."
  echo "Check 'docker ps' and 'docker logs configuration-osmosisd-1' (or matching container name)."
  exit 1
fi

echo ""
echo "Probing REST endpoint ${COSMOS_REST_URL}/cosmos/base/tendermint/v1beta1/node_info ..."
if curl -sf "${COSMOS_REST_URL}/cosmos/base/tendermint/v1beta1/node_info" > /dev/null 2>&1; then
  echo "  REST reachable."
else
  echo "  WARN: REST not reachable at ${COSMOS_REST_URL} (RPC is up; REST may need extra moments)."
fi

echo ""
echo "=== Cosmos-only bootstrap done ==="
echo "  RPC  : ${COSMOS_RPC_URL}"
echo "  REST : ${COSMOS_REST_URL}"
echo ""
echo "Next: 'make -C ci f0' (image build/push, contract deploy, lc-wasm upload, config patch)."
