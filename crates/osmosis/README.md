# stellar-osmosis

Bootstraps and manages a local Osmosis appchain (`localosmosis`) for stellar-ibc
devnets. It runs the prebuilt `osmolabs/osmosis:<ver>-alpine` image as a service
inside the repo-root `docker-compose.yml` and builds genesis from scratch through
a mounted entrypoint script — no Dockerfile, no source build, no caribic.

The chain config is **minimal and IBC-tailored**: it starts from `osmosisd init`
defaults and overrides only what relaying and the 08-wasm light client need —
`uosmo` as bond/mint/fee denom, a funded validator + relayer account, and a short
gov voting period + tiny deposit so the `stellaribc contracts upload-wasm`
governance proposal lands deterministically. It deliberately omits the LocalOsmosis
DEX-testing extras (denom metadata, balancer/stable/CL pools, incentive epochs).

## Layout

| File | Role |
|---|---|
| `assets/default-config.json` | Declarative chain config: chain id, moniker, genesis time, the `val`/`relayer` key mnemonics + their funded balances, the gentx, and the `genesis`/`app`/`config` override lists (each entry a `{path, type, value}` applied with `dasel`). Edit this, not the script. |
| `assets/setup.sh` | Container entrypoint. On first boot it `apk add jq dasel`, runs `osmosisd init`, applies every override from `default-config.json` (via `jq` + `dasel`), recovers each key and funds a genesis account at its derived address, builds the gentx, then `osmosisd start`. Data-driven — holds no hardcoded chain values. |
| `src/lifecycle.rs` | Locates the repo `docker-compose.yml` and drives `docker compose --profile osmosis up/down`. Resets `~/.osmosisd-local` for a fresh start unless `--stateful`. |
| `src/health.rs` | Polls `http://127.0.0.1:26658/status` until `latest_block_height > 0`. |
| `src/main.rs` | CLI: `start [--stateful]`, `stop`, `health`. |

The `osmosis` service definition lives in the repo-root `docker-compose.yml`
under the `osmosis` (and `local`) compose profiles.

## Usage

```sh
# via the orchestrator CLI (from anywhere in the repo)
stellaribc osmosis start       # start the devnet, wait for first block
stellaribc osmosis status
stellaribc osmosis stop

# or this crate directly
cargo run -p stellar-osmosis -- start [--stateful]   # --stateful keeps ~/.osmosisd-local
cargo run -p stellar-osmosis -- stop
cargo run -p stellar-osmosis -- health

# or straight through docker compose
docker compose --profile osmosis up -d osmosis
```

## Endpoints

| Endpoint | Host | Container |
|---|---|---|
| Tendermint RPC / websocket | `http://127.0.0.1:26658` | `26657` |
| REST (LCD) | `http://127.0.0.1:1318` | `1317` |
| gRPC | `127.0.0.1:9094` | `9090` |

Chain id `localosmosis`, account prefix `osmo`, gas denom `uosmo`. These match
`COSMOS_*` in `.env` and the `localosmosis` chain block in `ci/hermes-config.toml`.

Two keys are recovered into the genesis: `val` (the validator) and `relayer`
(a separately funded account for Hermes). Both mnemonics live in
`assets/default-config.json`. Point Hermes at the chain by importing the
`relayer` mnemonic under the `localosmosis` key name in `hermes-config.toml`:
`hermes keys add --chain localosmosis --mnemonic-file <relayer-mnemonic>`.

## Config

| Env var | Default | Effect |
|---|---|---|
| `OSMOSIS_VERSION` | `31.0.3` | `osmolabs/osmosis` image tag (the `-alpine` variant is used). |
| `OSMOSIS_LOCAL_GENESIS_TIME` | `2025-12-31T23:59:00Z` | `genesis_time` written into genesis. |
| `STELLAR_IBC_COMPOSE_FILE` | _(auto)_ | Override the compose file; otherwise the nearest `docker-compose.yml` above the cwd is used. |
| `OSMOSIS_CONFIG_JSON` | `/config/default-config.json` | Path (inside the container) to the chain config the entrypoint reads. |
