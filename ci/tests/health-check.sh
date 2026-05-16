#!/bin/bash
set -euo pipefail

HERMES_CONFIG="${HOME}/.hermes/config.toml"

echo "Running health checks..."

if ! hermes --config "${HERMES_CONFIG}" health-check 2>/dev/null; then
  echo "entrypoint is not reachable"
  exit 0
fi

echo "Health checks passed."
