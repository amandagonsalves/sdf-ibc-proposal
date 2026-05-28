#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
CI_DIR=$(cd "${SCRIPT_DIR}/.." && pwd)
REPO_ROOT=$(cd "${CI_DIR}/.." && pwd)

source "${SCRIPT_DIR}/_env.sh"
load_env_file "${REPO_ROOT}/.env"

API_IMAGE="${API_IMAGE:-amandagonsalvesx/stellar-ibc-api}"
API_TAG="${API_TAG:-latest}"
API_REGISTRY="${API_REGISTRY:-}"
PUSH="${PUSH:-1}"
DOCKERFILE="${REPO_ROOT}/Dockerfile"

if ! command -v docker > /dev/null 2>&1; then
  echo "ERROR: docker not found in PATH."
  exit 1
fi

if [[ ! -f "${DOCKERFILE}" ]]; then
  echo "ERROR: Dockerfile missing at ${DOCKERFILE}"
  exit 1
fi

if ! grep -q "stellar-api" "${DOCKERFILE}"; then
  echo "ERROR: ${DOCKERFILE} does not look like the api Dockerfile (no 'stellar-api' reference)."
  exit 1
fi

if [[ -n "${API_REGISTRY}" ]]; then
  LOCAL_REF="${API_IMAGE}:${API_TAG}"
  REMOTE_REF="${API_REGISTRY}/${API_IMAGE}:${API_TAG}"
else
  LOCAL_REF="${API_IMAGE}:${API_TAG}"
  REMOTE_REF="${LOCAL_REF}"
fi

echo "=== build-api-image ==="
echo "  Context      : ${REPO_ROOT}"
echo "  Dockerfile   : ${DOCKERFILE}"
echo "  Local ref    : ${LOCAL_REF}"
echo "  Remote ref   : ${REMOTE_REF}"
echo "  Push enabled : ${PUSH}"
echo ""

echo "Step 1: docker build -t ${LOCAL_REF} -f ${DOCKERFILE} ${REPO_ROOT}"
docker build \
  -t "${LOCAL_REF}" \
  -f "${DOCKERFILE}" \
  "${REPO_ROOT}"

echo ""
echo "Step 2: smoke-test the api binary starts cleanly (3s probe)"
ENV_ARG=()
if [[ -f "${REPO_ROOT}/.env" ]]; then
  ENV_ARG=(--env-file "${REPO_ROOT}/.env")
fi
CID=$(docker run -d --rm --entrypoint stellar-api "${ENV_ARG[@]}" "${LOCAL_REF}")
sleep 3
if docker ps -q -f "id=${CID}" | grep -q .; then
  echo "  Container ${CID:0:12} still running after 3s — startup OK."
  docker logs "${CID}" 2>&1 | tail -5 || true
  docker stop "${CID}" > /dev/null 2>&1 || true
else
  echo "  ERROR: container ${CID:0:12} exited within 3s."
  docker logs "${CID}" 2>&1 | tail -20 || true
  exit 1
fi

if [[ "${LOCAL_REF}" != "${REMOTE_REF}" ]]; then
  echo ""
  echo "Step 3: tagging as ${REMOTE_REF}"
  docker tag "${LOCAL_REF}" "${REMOTE_REF}"
fi

if [[ "${PUSH}" == "1" || "${PUSH}" == "true" ]]; then
  echo ""
  echo "Step 4a: docker login (from .env credentials if set)"
  docker_login_if_creds "${PUSH}"

  echo ""
  echo "Step 4b: docker push ${REMOTE_REF}"
  if ! docker push "${REMOTE_REF}"; then
    echo ""
    echo "ERROR: push failed. Likely causes:"
    echo "  - DOCKER_USERNAME/DOCKER_TOKEN missing or wrong in .env"
    echo "  - API_IMAGE (${API_IMAGE}) does not match your DockerHub namespace"
    echo "  - repo does not exist on DockerHub (create it via the UI first)"
    exit 1
  fi
else
  echo ""
  echo "Step 4: SKIP push (PUSH=${PUSH}). Image lives locally as ${REMOTE_REF}."
fi

echo ""
echo "=== build-api-image done ==="
echo "  Image : ${REMOTE_REF}"
echo ""
echo "Run locally   : docker run --rm --env-file .env -p 8101:8101 --entrypoint stellar-api ${REMOTE_REF}"
if [[ "${PUSH}" == "1" || "${PUSH}" == "true" ]]; then
  echo "Pull anywhere : docker pull ${REMOTE_REF}"
fi
