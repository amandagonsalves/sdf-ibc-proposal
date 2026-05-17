# ci/ — Local integration tests for stellar-ibc

## What gets tested

### Gateway + Hermes integration (T-1 through T-5)

Run against a live Stellar gateway and Hermes fork. Start the gateway first:

```bash
caribic chain start --chain stellar
bash ci/hermes-health.sh
# or: make -C ci hermes-health
```

| Test | What it checks |
|---|---|
| T-1 | Gateway reachable: gRPC :50052, HTTP :8001, soroban-testnet.stellar.org:443 |
| T-2 | `hermes health-check` — stellar-testnet, cardano-devnet, cardano-entrypoint all healthy |
| T-3 | `LatestHeight` gRPC returns `revisionHeight > 0` |
| T-4 | `~/.hermes/config.toml` contains stellar-testnet block with `type = 'Stellar'` |
| T-5 | `hermes create client` — currently BLOCKED on `QueryIbcHeader` (shown, not a failure) |

### Docker integration (D-1 through D-9)

Builds the gateway image, starts it via Docker Compose, verifies all checks, then tears down.
No gateway needs to be running beforehand — the script manages the full lifecycle.

```bash
# Testnet profile (default — gateway points at soroban-testnet.stellar.org)
bash ci/docker-health.sh
# or: make -C ci docker-health

# Local node profile (starts stellar/quickstart alongside the gateway)
PROFILE=local bash ci/docker-health.sh
# or: make -C ci docker-health-local
```

| Test | Profile | What it checks |
|---|---|---|
| D-1 | both | `stellar-gateway` container appears in `docker compose ps` |
| D-2 | both | `GET /health` → `"Server is up."` on HTTP `:8001` |
| D-3 | both | gRPC port `:50052` accepting connections |
| D-4 | both | `LatestHeight` gRPC returns `revisionHeight > 0` |
| D-5 | both | Docker healthcheck reports container as `healthy` |
| D-6 | both | gRPC reflection lists `stellar.gateway.v1.StellarGatewayQuery` |
| D-7 | local | Stellar node `GET /health` on `:8000` returns OK |
| D-8 | local | Gateway HTTP health after connecting to local node |
| D-9 | local | `LatestHeight` gRPC > 0 against local node |

### WASM light client (against Cosmos entrypoint)

| Test | Requires chain | What it checks |
|---|---|---|
| `tests/health-check.sh` | Yes | Hermes can reach `cardano-entrypoint` on `localhost:26657` |
| `tests/upload-wasm.sh` | Yes | WASM uploads via `08-wasm`, checksum appears on-chain |

Tests skip automatically (exit 0) when the chain is not reachable, so `entrypoint.sh`
always exits cleanly whether or not the chain is running.

## Prerequisites

**1. Hermes** — install the pre-built binary (building from source is broken):

```bash
# macOS Apple Silicon
curl -L https://github.com/informalsystems/hermes/releases/download/v1.13.2/hermes-v1.13.2-aarch64-apple-darwin.tar.gz \
  | tar -xz && mkdir -p ~/.local/bin && mv hermes ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"

# macOS: allow it through Gatekeeper if prompted
xattr -d com.apple.quarantine ~/.local/bin/hermes
```

**2. wasm32 target** for Rust:

```bash
rustup target add wasm32-unknown-unknown
```

**3. Cosmos entrypoint chain** (from `cardano-ibc-incubator`) — needed only for
the upload test, not for the WASM build:

```bash
# In a separate terminal, inside cardano-ibc-incubator/
cd cosmos/cardano-entrypoint
ignite chain serve -y
# Wait until: 🌍 Tendermint node: http://0.0.0.0:26657
```

## Running locally

From the `stellar-ibc/` root:

```bash
bash ci/entrypoint.sh
```

This will:
1. Build `stellar-ibc-light-client` for `wasm32-unknown-unknown`
2. Write `ci/hermes-config.toml` → `~/.hermes/config.toml`
3. Add the devnet relayer key to Hermes from `ci/relayer-mnemonic.txt`
4. Run all tests in `ci/tests/`

## Relayer mnemonic

`ci/relayer-mnemonic.txt` contains the `relayer` account mnemonic from
`cardano-ibc-incubator/cosmos/cardano-entrypoint/config.yml`. This is a local
devnet account pre-funded with `10000token` and `100000000stake` in genesis —
enough to pay for the WASM upload transaction.

To use a different key:

```bash
RELAYER_MNEMONIC="your twelve word mnemonic here ..." bash ci/entrypoint.sh
```

## File structure

```
ci/
  entrypoint.sh          Main script: build → configure → key → tests
  hermes-health.sh       T-1…T-5 gateway + hermes integration tests
  docker-health.sh       D-1…D-9 Docker Compose lifecycle tests
  hermes-config.toml     Hermes config pointing at localhost:26657
  relayer-mnemonic.txt   Devnet relayer mnemonic (from incubator config.yml)
  Makefile               Individual test targets
  tests/
    health-check.sh      hermes health-check (entrypoint sub-test)
    upload-wasm.sh       hermes client store wasm-code + checksum verification
```
