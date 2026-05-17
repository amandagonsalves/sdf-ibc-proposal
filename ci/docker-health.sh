#!/usr/bin/env bash
# Docker integration tests for stellar-ibc.
# Builds the gateway image and runs the testnet profile, verifies all checks,
# then tears down. Optionally tests the local-node profile too.
#
# Usage:
#   bash ci/docker-health.sh              # testnet profile only (default)
#   PROFILE=local bash ci/docker-health.sh  # also test local Stellar node profile
#
# Exit 0 — all tests passed
# Exit 1 — one or more tests failed

set -uo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)

GRPC_PORT="${STELLAR_GATEWAY_GRPC_PORT:-50052}"
HTTP_PORT="${STELLAR_GATEWAY_HTTP_PORT:-8001}"
PROFILE="${PROFILE:-testnet}"
COMPOSE_FILE="${REPO_ROOT}/docker-compose.yml"

PASS=0
FAIL=0

# ── Helpers ───────────────────────────────────────────────────────────────────

pass()   { echo "    [PASS] $1"; PASS=$((PASS + 1)); }
fail()   { echo "    [FAIL] $1"; FAIL=$((FAIL + 1)); }
header() { echo ""; echo "── $1 ──────────────────────────────────────────────"; }

check_prereq() {
  if ! command -v "$1" &>/dev/null; then
    echo "ERROR: '$1' not found. Install it before running this script."
    exit 1
  fi
}

wait_for_port() {
  local host="$1" port="$2" label="$3" max_wait="${4:-60}"
  echo "  Waiting for ${label} on ${host}:${port} (up to ${max_wait}s)..."
  local elapsed=0
  while ! nc -z "$host" "$port" 2>/dev/null; do
    sleep 2; elapsed=$((elapsed + 2))
    if [[ $elapsed -ge $max_wait ]]; then
      echo "  ERROR: ${label} not ready after ${max_wait}s"
      return 1
    fi
  done
  echo "  ${label} is up (${elapsed}s)"
}

compose_down() {
  local profile_args=()
  [[ "${1:-}" == "local" ]] && profile_args=(--profile local)
  docker compose -f "${COMPOSE_FILE}" "${profile_args[@]}" down --remove-orphans 2>/dev/null || true
}

# ── Prerequisites ─────────────────────────────────────────────────────────────

check_prereq docker
check_prereq grpcurl

if ! docker compose version &>/dev/null 2>&1; then
  echo "ERROR: 'docker compose' (v2) not found. Update Docker Desktop."
  exit 1
fi

echo "Stellar IBC — Docker integration tests"
echo "======================================="
echo "  Repo:     ${REPO_ROOT}"
echo "  Profile:  ${PROFILE}"
echo "  gRPC port: ${GRPC_PORT}  HTTP port: ${HTTP_PORT}"

# ── Testnet profile ───────────────────────────────────────────────────────────

header "Testnet profile  (docker compose up)"

echo "  Tearing down any previous containers..."
compose_down
echo ""

# Set a dummy signing key so the gateway starts without crashing.
# The gateway does not need a real key for health checks.
STELLAR_SIGNING_KEY="${STELLAR_SIGNING_KEY:-SCZANGBA5AKIA4MKQHKROLIOA7JJXZC4WVQJWMF4AEKF6XKMJKF6YH3}"

echo "  Building and starting gateway (testnet)..."
STELLAR_SIGNING_KEY="${STELLAR_SIGNING_KEY}" \
STELLAR_GATEWAY_GRPC_PORT="${GRPC_PORT}" \
STELLAR_GATEWAY_HTTP_PORT="${HTTP_PORT}" \
  docker compose -f "${COMPOSE_FILE}" up -d --build 2>&1 | tail -5

# D-1: Container is running
header "D-1  stellar-gateway container is running"
sleep 3
CONTAINER_STATE=$(docker compose -f "${COMPOSE_FILE}" ps --format json 2>/dev/null \
  | grep -o '"stellar-ibc-stellar-gateway[^"]*"\|"stellar_gateway[^"]*"' | head -1 || true)
RAW_PS=$(docker compose -f "${COMPOSE_FILE}" ps 2>/dev/null)
echo "${RAW_PS}"
if echo "${RAW_PS}" | grep -q "stellar-gateway"; then
  pass "stellar-gateway container exists"
else
  fail "stellar-gateway container not found in docker compose ps"
fi

# D-2: HTTP health endpoint
header "D-2  HTTP health endpoint  (GET /health)"
if wait_for_port localhost "${HTTP_PORT}" "HTTP server" 60; then
  HTTP_BODY=$(curl -sf "http://localhost:${HTTP_PORT}/health" 2>/dev/null || echo "CURL_FAILED")
  echo "  Response: ${HTTP_BODY}"
  if [[ "${HTTP_BODY}" == "Server is up." ]]; then
    pass "GET /health → \"Server is up.\""
  else
    fail "unexpected response: ${HTTP_BODY}"
  fi
else
  fail "HTTP server did not come up within 60s"
fi

# D-3: gRPC port reachable
header "D-3  gRPC port reachable  (:${GRPC_PORT})"
if nc -z localhost "${GRPC_PORT}" 2>/dev/null; then
  pass "gRPC :${GRPC_PORT} reachable"
else
  fail "gRPC :${GRPC_PORT} not reachable"
fi

# D-4: LatestHeight gRPC
header "D-4  LatestHeight gRPC"
GRPC_OUT=$(grpcurl -plaintext "localhost:${GRPC_PORT}" \
  stellar.gateway.v1.StellarGatewayQuery/LatestHeight 2>&1 || true)
echo "  Response: ${GRPC_OUT}"
HEIGHT=$(echo "${GRPC_OUT}" | grep -o '"revisionHeight": *"[0-9]*"' | grep -o '[0-9]*' || true)
if [[ -n "${HEIGHT}" && "${HEIGHT}" -gt 0 ]]; then
  pass "revisionHeight = ${HEIGHT} (> 0)"
else
  fail "revisionHeight not returned or zero"
fi

# D-5: Docker healthcheck passes
header "D-5  Docker healthcheck reports healthy"
sleep 15
HEALTH=$(docker inspect --format '{{.State.Health.Status}}' \
  "$(docker compose -f "${COMPOSE_FILE}" ps -q stellar-gateway 2>/dev/null)" 2>/dev/null || echo "unknown")
echo "  Health status: ${HEALTH}"
if [[ "${HEALTH}" == "healthy" ]]; then
  pass "container health = healthy"
else
  fail "container health = ${HEALTH} (expected healthy)"
fi

# D-6: gRPC reflection lists StellarGatewayQuery
header "D-6  gRPC reflection lists StellarGatewayQuery"
SERVICES=$(grpcurl -plaintext "localhost:${GRPC_PORT}" list 2>/dev/null || true)
echo "  Services: ${SERVICES}"
if echo "${SERVICES}" | grep -q "stellar.gateway.v1.StellarGatewayQuery"; then
  pass "StellarGatewayQuery present in reflection"
else
  fail "StellarGatewayQuery not in gRPC reflection"
fi

echo ""
echo "  Stopping testnet containers..."
compose_down

# ── Local node profile (optional) ─────────────────────────────────────────────

if [[ "${PROFILE}" == "local" ]]; then
  header "Local node profile  (docker compose --profile local up)"
  echo "  Starting stellar-node + stellar-gateway (local)..."
  STELLAR_SIGNING_KEY="${STELLAR_SIGNING_KEY}" \
  STELLAR_RPC_URL="http://stellar-node:8000/soroban/rpc" \
  NETWORK_PASSPHRASE="Standalone Network ; February 2017" \
  STELLAR_GATEWAY_GRPC_PORT="${GRPC_PORT}" \
  STELLAR_GATEWAY_HTTP_PORT="${HTTP_PORT}" \
    docker compose -f "${COMPOSE_FILE}" --profile local up -d --build 2>&1 | tail -5

  header "D-7  Local Stellar node HTTP reachable  (:8000)"
  if wait_for_port localhost 8000 "Stellar node" 120; then
    NODE_HEALTH=$(curl -sf "http://localhost:8000/health" 2>/dev/null || echo "CURL_FAILED")
    echo "  Response: ${NODE_HEALTH}"
    if echo "${NODE_HEALTH}" | grep -q "healthy\|ok\|true"; then
      pass "Stellar node /health OK"
    else
      fail "unexpected node health response: ${NODE_HEALTH}"
    fi
  else
    fail "Stellar node did not come up within 120s"
  fi

  header "D-8  Gateway HTTP health (local node mode)"
  if wait_for_port localhost "${HTTP_PORT}" "gateway HTTP" 60; then
    HTTP_BODY=$(curl -sf "http://localhost:${HTTP_PORT}/health" 2>/dev/null || echo "CURL_FAILED")
    echo "  Response: ${HTTP_BODY}"
    [[ "${HTTP_BODY}" == "Server is up." ]] && pass "GET /health OK" || fail "unexpected: ${HTTP_BODY}"
  else
    fail "gateway did not come up within 60s"
  fi

  header "D-9  LatestHeight gRPC (local node)"
  GRPC_OUT=$(grpcurl -plaintext "localhost:${GRPC_PORT}" \
    stellar.gateway.v1.StellarGatewayQuery/LatestHeight 2>&1 || true)
  echo "  Response: ${GRPC_OUT}"
  HEIGHT=$(echo "${GRPC_OUT}" | grep -o '"revisionHeight": *"[0-9]*"' | grep -o '[0-9]*' || true)
  if [[ -n "${HEIGHT}" && "${HEIGHT}" -gt 0 ]]; then
    pass "revisionHeight = ${HEIGHT} (> 0, local node)"
  else
    fail "revisionHeight not returned or zero"
  fi

  echo ""
  echo "  Stopping local containers..."
  compose_down local
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "======================================="
printf "Results: %d passed  %d failed\n" "$PASS" "$FAIL"
echo "======================================="

[[ $FAIL -eq 0 ]] || exit 1
