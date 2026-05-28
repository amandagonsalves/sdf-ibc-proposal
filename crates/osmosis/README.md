# stellar-osmosis

Bootstraps and manages a local Osmosis appchain (`localosmosis`) for stellar-ibc
devnets. This is the stellar-ibc port of caribic's `caribic chain start --chain
osmosis --network local`: instead of downloading the Osmosis source and building
`osmosisd` from a Dockerfile, it runs the prebuilt `osmolabs/osmosis:<ver>-alpine`
image as a service inside the repo-root `docker-compose.yml`, initialising genesis
through a mounted entrypoint script.

## Layout

| File | Role |
|---|---|
| `assets/default-config.json` | Declarative chain config: chain id, moniker, genesis time, validator/pools mnemonics, funded genesis accounts, and the `genesis`/`app`/`config` override lists (each entry is a `{path, type, value}` applied with `dasel`). Edit this rather than the script. Ported from caribic's `chains/osmosis/scripts/setup_osmosis_local.sh` (itself adapted from upstream Osmosis `tests/localosmosis/scripts/setup.sh`). |
| `assets/setup.sh` | Container entrypoint. On first boot it `apk add jq dasel`, runs `osmosisd init`, then applies every override from `default-config.json` (via `jq` + `dasel`), funds genesis accounts, builds the gentx, and `osmosisd start`. Data-driven — it holds no hardcoded chain values. Mounted into the `osmosis` service alongside the config. |
| `src/lifecycle.rs` | Locates the repo `docker-compose.yml` and drives `docker compose --profile osmosis up/down`. Resets `~/.osmosisd-local` for a fresh start unless `--stateful`. |
| `src/health.rs` | Polls `http://127.0.0.1:26658/status` until `latest_block_height > 0`. |
| `src/main.rs` | CLI: `start [--stateful]`, `stop`, `health`. |

The `osmosis` service definition lives in the repo-root `docker-compose.yml`
under the `osmosis` (and `local`) compose profiles.

## Usage

```sh
make start-osmosis            # fresh local chain, wait for first block
make start-osmosis-stateful   # reuse existing ~/.osmosisd-local state
make health-osmosis
make stop-osmosis

# or directly
cargo run -p stellar-osmosis -- start [--stateful]
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
| gRPC-web | `127.0.0.1:9091` | `9091` |

Chain id `localosmosis`, account prefix `osmo`, gas denom `uosmo`. These match
`COSMOS_*` in `.env` and the `localosmosis` chain block in `ci/hermes-config.toml`.

The validator (`val`) and `pools` key mnemonics used for the funded genesis
accounts are in `assets/setup.sh`; import them with
`osmosisd keys add <name> --recover` or `hermes keys add` to fund a relayer.

## Config

| Env var | Default | Effect |
|---|---|---|
| `OSMOSIS_VERSION` | `31.0.3` | `osmolabs/osmosis` image tag (the `-alpine` variant is used). |
| `OSMOSIS_LOCAL_GENESIS_TIME` | `2025-12-31T23:59:00Z` | `genesis_time` written into genesis. |
| `STELLAR_IBC_COMPOSE_FILE` | _(auto)_ | Override the compose file; otherwise the nearest `docker-compose.yml` above the cwd is used. |
| `OSMOSIS_CONFIG_JSON` | `/config/default-config.json` | Path (inside the container) to the chain config the entrypoint reads. |
