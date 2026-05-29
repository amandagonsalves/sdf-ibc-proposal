#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
CI_DIR=$(cd "${SCRIPT_DIR}/.." && pwd)
REPO_ROOT=$(cd "${CI_DIR}/.." && pwd)

source "${SCRIPT_DIR}/_env.sh"
load_env_file "${REPO_ROOT}/.env"

CHAIN_ID="${COSMOS_CHAIN_ID:-localosmosis}"
COSMOS_REST="${COSMOS_REST_URL:-http://127.0.0.1:1318}"
GATEWAY_HTTP="${GATEWAY_HTTP:-http://127.0.0.1:${STELLAR_GATEWAY_HTTP_PORT:-8101}}"
GATEWAY_GRPC="${GATEWAY_GRPC:-127.0.0.1:${STELLAR_GATEWAY_GRPC_PORT:-50052}}"
HERMES_CONFIG="${HERMES_CONFIG:-${CI_DIR}/hermes-config.toml}"

echo "=== F0: bootstrap (images + chain probes + Soroban contract deploy + lc-wasm upload + hermes config patch) ==="

if [[ "${SKIP_IMAGE_BUILD:-0}" != "1" ]]; then
  echo ""
  echo "Step 0a: Building + pushing stellar-gateway docker image..."
  bash "${SCRIPT_DIR}/build-gateway-image.sh"

  echo ""
  echo "Step 0b: Building + pushing hermes docker image..."
  bash "${SCRIPT_DIR}/build-hermes-image.sh"
else
  echo ""
  echo "Step 0: SKIP image build (SKIP_IMAGE_BUILD=1)."
fi

echo ""
echo "Step 1: Probing Cosmos ${CHAIN_ID} REST at ${COSMOS_REST}..."
if ! curl -sf "${COSMOS_REST}/cosmos/base/tendermint/v1beta1/node_info" > /dev/null 2>&1; then
  echo "  SKIP: ${CHAIN_ID} not reachable."
  echo "  Start it with: make -C ci cosmos-only"
  exit 0
fi
echo "  Reachable."

echo ""
echo "Step 2: Probing Stellar gateway at ${GATEWAY_HTTP}/health..."
if ! curl -sf "${GATEWAY_HTTP}/health" > /dev/null 2>&1; then
  echo "  SKIP: stellar-hermes-gateway not reachable at ${GATEWAY_HTTP}."
  echo "  Start it with: cargo run -p stellar-hermes-gateway"
  exit 0
fi
echo "  Reachable. gRPC expected at ${GATEWAY_GRPC}."

echo ""
echo "================================================================="
echo "Step 3: Soroban contracts — build + upload + deploy + wire router"
echo "================================================================="
if [[ "${SKIP_CONTRACT_DEPLOY:-0}" != "1" ]]; then
  bash "${SCRIPT_DIR}/upload-and-deploy-contracts.sh"
  load_env_file "${REPO_ROOT}/.env"

  if command -v docker > /dev/null 2>&1 \
      && docker compose --profile local --profile hermes ps -q gateway 2>/dev/null | grep -q .; then
    echo ""
    echo "  Restarting gateway so it picks up the new IBC_CONTRACT_ID..."
    docker compose --profile local --profile hermes rm -sf gateway > /dev/null
    docker compose --profile local --profile hermes up -d gateway > /dev/null
    echo "  Gateway recreated."
  else
    echo ""
    echo "  Gateway container not found via docker compose. If you run the gateway"
    echo "  on the host, restart it manually so it reads the new IBC_CONTRACT_ID."
  fi
else
  echo "  SKIP contract deploy (SKIP_CONTRACT_DEPLOY=1)."
fi

echo ""
echo "Step 4: Upload light-client-wasm + patch hermes config..."
if [[ "${SKIP_LC_WASM_UPLOAD:-0}" != "1" ]]; then
  bash "${SCRIPT_DIR}/upload-lc-wasm.sh"
else
  echo "  SKIP lc-wasm upload (SKIP_LC_WASM_UPLOAD=1)."
fi

load_env_file "${REPO_ROOT}/.env"

echo ""
echo "=== F0 done ==="
echo "  Cosmos chain    : ${CHAIN_ID} (reachable)"
echo "  Stellar GW      : ${GATEWAY_HTTP} (reachable, gRPC ${GATEWAY_GRPC})"
echo "  Hermes config   : ${HERMES_CONFIG}"
echo "  Deployer addr   : ${DEPLOYER_ADDRESS:-(unset)}"
echo "  IBC router      : ${IBC_CONTRACT_ID:-(unset)}"
echo "  Transfer app    : ${TRANSFER_CONTRACT_ID:-(unset)}"
echo "  Mock LC         : ${MOCK_LC_CONTRACT_ID:-(unset)}"
[[ -n "${ATTESTATION_LC_CONTRACT_ID:-}" ]] && echo "  Attestation LC  : ${ATTESTATION_LC_CONTRACT_ID}"
[[ -n "${TENDERMINT_LC_CONTRACT_ID:-}" ]]  && echo "  Tendermint LC   : ${TENDERMINT_LC_CONTRACT_ID}"
echo ""
echo "Next: import hermes keys (make hermes-keys) and create clients."
