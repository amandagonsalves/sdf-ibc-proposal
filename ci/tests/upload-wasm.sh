#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)
WASM_FILE="${REPO_ROOT}/target/wasm32-unknown-unknown/release/stellar_ibc_light_client.wasm"
CONTAINER="cardano-entrypoint-node-prod"
CHAIN_BIN="/go/bin/cardano-entrypointd"
CHAIN_HOME="/root/.cardano-entrypoint-data/node"
CHAIN_ID="cardanoentrypoint"
NODE="tcp://localhost:26657"
REST="http://localhost:1317"
VOTING_PERIOD=20   # seconds — must exceed gov.params.voting_period in config.yml (15s)

echo "=== 08-wasm upload test ==="

if [[ ! -f "${WASM_FILE}" ]]; then
  echo "ERROR: WASM not found at ${WASM_FILE}"
  echo "  Run: bash ci/entrypoint.sh"
  exit 1
fi

echo "WASM: ${WASM_FILE} ($(wc -c < "${WASM_FILE}") bytes)"

echo ""
echo "Step 1: Checking ${CHAIN_ID} is reachable..."
if ! curl -sf "${REST}/cosmos/base/tendermint/v1beta1/node_info" > /dev/null 2>&1; then
  echo "SKIP: ${CHAIN_ID} is not reachable at ${REST}."
  echo "  Start it with: caribic start --clean"
  exit 0
fi
echo "  Chain is up."

echo ""
echo "Step 2: Checking Docker container..."
if ! docker inspect "${CONTAINER}" > /dev/null 2>&1; then
  echo "SKIP: Container '${CONTAINER}' not found. Is caribic running?"
  exit 0
fi
echo "  Container: ${CONTAINER}"

echo ""
echo "Step 3: Copying WASM into container..."
docker cp "${WASM_FILE}" "${CONTAINER}:/tmp/stellar_ibc_light_client.wasm"
echo "  Copied to /tmp/stellar_ibc_light_client.wasm"

TX_FLAGS="--keyring-backend test --home ${CHAIN_HOME} --chain-id ${CHAIN_ID} --node ${NODE} --gas auto --gas-adjustment 1.4 -y -o json"

echo ""
echo "Step 4: Submitting governance proposal to store WASM..."
PROPOSAL_OUTPUT=$(docker exec "${CONTAINER}" \
  "${CHAIN_BIN}" tx ibc-wasm store-code /tmp/stellar_ibc_light_client.wasm \
  --from relayer \
  --title "Upload Stellar IBC light client" \
  --summary "Registers stellar_ibc_light_client.wasm as client type 10-stellar" \
  --deposit "1stake" \
  ${TX_FLAGS} 2>&1) || {
  echo "ERROR: governance proposal submission failed:"
  echo "${PROPOSAL_OUTPUT}"
  exit 1
}

echo "${PROPOSAL_OUTPUT}"
echo "  Waiting for proposal tx to be included..."
sleep 4

echo ""
echo "Step 5: Finding proposal ID..."
PROPOSAL_ID=$(curl -sf "${REST}/cosmos/gov/v1/proposals?proposal_status=PROPOSAL_STATUS_VOTING_PERIOD" 2>/dev/null \
  | python3 -c "import sys,json; ps=json.load(sys.stdin).get('proposals',[]); print(ps[-1]['id'] if ps else '')" 2>/dev/null || true)

if [[ -z "${PROPOSAL_ID}" ]]; then
  echo "ERROR: Could not find proposal in voting period."
  exit 1
fi
echo "  Proposal ID: ${PROPOSAL_ID}"

echo ""
echo "Step 6: Voting YES with alice (validator)..."
docker exec "${CONTAINER}" \
  "${CHAIN_BIN}" tx gov vote "${PROPOSAL_ID}" yes \
  --from alice \
  ${TX_FLAGS} > /dev/null 2>&1

echo "  Voted YES. Waiting ${VOTING_PERIOD}s for voting period to end..."
sleep ${VOTING_PERIOD}

echo ""
echo "Step 7: Verifying checksum is registered on-chain..."
CHECKSUMS=$(docker exec "${CONTAINER}" \
  "${CHAIN_BIN}" query ibc-wasm checksums \
  --node "${NODE}" -o json 2>&1) || true

echo "${CHECKSUMS}"

if echo "${CHECKSUMS}" | python3 -c "import sys,json; cs=json.load(sys.stdin).get('checksums',[]); exit(0 if cs else 1)" 2>/dev/null; then
  echo ""
  echo "SUCCESS: Stellar light client WASM uploaded and registered."
  echo "  The chain can now create 10-stellar clients via 08-wasm."
else
  echo "ERROR: No checksums found. Proposal may not have passed."
  echo "  Check proposal status: ${REST}/cosmos/gov/v1/proposals/${PROPOSAL_ID}"
  exit 1
fi
