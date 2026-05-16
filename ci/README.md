# ci/ — Local integration tests for stellar-ibc

Tests the Stellar IBC light client WASM against a live Cosmos entrypoint chain.

## What gets tested

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
  hermes-config.toml     Hermes config pointing at localhost:26657
  relayer-mnemonic.txt   Devnet relayer mnemonic (from incubator config.yml)
  Makefile               Individual test targets (upload-wasm, health-check)
  tests/
    health-check.sh      hermes health-check
    upload-wasm.sh       hermes client store wasm-code + checksum verification
```
