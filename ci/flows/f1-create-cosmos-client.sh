#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
CI_DIR=$(cd "${SCRIPT_DIR}/.." && pwd)
REPO_ROOT=$(cd "${CI_DIR}/.." && pwd)
ENV_FILE="${REPO_ROOT}/.env"

source "${SCRIPT_DIR}/_env.sh"
load_env_file "${ENV_FILE}"

HERMES_SERVICE="${HERMES_SERVICE:-hermes}"
HERMES_CONFIG_IN_CONTAINER="${HERMES_CONFIG_IN_CONTAINER:-/home/hermes/.hermes/config.toml}"
COSMOS_RPC_URL="${COSMOS_RPC_URL:-http://127.0.0.1:26658}"
GATEWAY_GRPC_PORT="${STELLAR_GATEWAY_GRPC_PORT:-50052}"
FORCE_RECREATE="${FORCE_RECREATE:-0}"

HOST_CHAIN="stellar-testnet"
REFERENCE_CHAIN="localosmosis"
RESULT_ENV_VAR="COSMOS_CLIENT_ID"

echo "=== f1-create-cosmos-client ==="
echo "  Host chain    : ${HOST_CHAIN} (where the client lives)"
echo "  Reference     : ${REFERENCE_CHAIN} (the Cosmos chain it tracks)"
echo "  Result var    : ${RESULT_ENV_VAR}"
echo ""

EXISTING="${!RESULT_ENV_VAR:-}"
if [[ -n "${EXISTING}" && "${FORCE_RECREATE}" != "1" && "${FORCE_RECREATE}" != "true" ]]; then
  echo "  ${RESULT_ENV_VAR} already set in .env: ${EXISTING}"
  echo "  Nothing to do. Set FORCE_RECREATE=1 to create another client."
  exit 0
fi

if ! command -v docker > /dev/null 2>&1; then
  echo "  ERROR: docker not found in PATH."
  exit 1
fi

echo "Step 1: Probing gateway gRPC at 127.0.0.1:${GATEWAY_GRPC_PORT}..."
if ! (exec 3<>"/dev/tcp/127.0.0.1/${GATEWAY_GRPC_PORT}") 2>/dev/null; then
  echo "  SKIP: gateway not reachable. Start the stack with: docker compose --profile staging up -d gateway api osmosis"
  exit 0
fi
echo "  Reachable."

echo ""
echo "Step 2: Probing Cosmos RPC at ${COSMOS_RPC_URL}/status..."
if ! curl -sf --max-time 10 "${COSMOS_RPC_URL}/status" > /dev/null 2>&1; then
  echo "  SKIP: Cosmos RPC not reachable. Start it with: make -C ci cosmos-only"
  exit 0
fi
echo "  Reachable."

echo ""
echo "Step 3: hermes create client --host-chain ${HOST_CHAIN} --reference-chain ${REFERENCE_CHAIN}"
cd "${REPO_ROOT}"
CREATE_OUTPUT=$(docker compose run --rm "${HERMES_SERVICE}" \
  --config "${HERMES_CONFIG_IN_CONTAINER}" \
  create client \
  --host-chain "${HOST_CHAIN}" \
  --reference-chain "${REFERENCE_CHAIN}" \
  2>&1) || {
  echo "ERROR: hermes create client failed:"
  echo "${CREATE_OUTPUT}"
  exit 1
}
echo "${CREATE_OUTPUT}"

CLIENT_ID=$(echo "${CREATE_OUTPUT}")
if [[ -z "${CLIENT_ID}" ]]; then
  echo "ERROR: no client id found in hermes output."
  exit 1
fi
echo "  Created: ${CLIENT_ID}"

echo ""
echo "Step 4: writing ${RESULT_ENV_VAR}=${CLIENT_ID} into ${ENV_FILE}..."
python3 - "${ENV_FILE}" "${RESULT_ENV_VAR}=${CLIENT_ID}" <<'PY'
import sys, re, pathlib
path = pathlib.Path(sys.argv[1])
key, value = sys.argv[2].split("=", 1)
text = path.read_text()
pattern = re.compile(rf"^{re.escape(key)}\s*=.*$", re.MULTILINE)
if pattern.search(text):
    text = pattern.sub(f"{key}={value}", text)
else:
    if not text.endswith("\n"):
        text += "\n"
    text += f"{key}={value}\n"
path.write_text(text)
print(f"  {key}={value}")
PY

echo ""
echo "=== f1-create-cosmos-client done ==="
echo "  ${RESULT_ENV_VAR} : ${CLIENT_ID}"
echo "  ${HOST_CHAIN} now tracks ${REFERENCE_CHAIN}."
echo "  Next: register the counterparty (F1.3) so the two clients are paired."
