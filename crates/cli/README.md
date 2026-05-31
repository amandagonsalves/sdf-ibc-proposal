# `stellaribc` — Stellar↔Cosmos IBC orchestrator CLI

A front door to the `ci/flows/*.sh` scripts. One binary instead
of remembering script names and skip flags. It discovers the repo root
(walking up for `ci/flows/_env.sh`, or `STELLAR_IBC_ROOT`), loads `.env`
declaratively, probes service health natively, and delegates the actual work
to the canonical flow scripts (one source of truth).

## Install / run

```sh
# run in-place
cargo run -p stellar-ibc-cli -- <command>

# or install the `stellaribc` binary
cargo install --path crates/cli
stellaribc <command>
```

## Commands

| Command | Wraps | What it does |
|---|---|---|
| `stellaribc doctor` | — | check toolchain, `.env`, config, and service health |
| `stellaribc status` | — | probe chains/services, show contract ids + created clients |
| `stellaribc up [--cosmos\|--stellar]` | `docker compose` | start osmosis + api + gateway (or one side) |
| `stellaribc down [--volumes]` | `docker compose` | stop the stack |
| `stellaribc bootstrap` (alias `f0`) | `f0-bootstrap.sh` | full F0; `--skip-images/-contracts/-wasm/-keys`, `--force-redeploy` |
| `stellaribc build-images [all\|api\|gateway\|hermes]` | `build-*-image.sh` | build + push the docker images |
| `stellaribc deploy-contracts [--force]` | `upload-and-deploy-contracts.sh` | build/upload/deploy + wire the router |
| `stellaribc upload-wasm` | `upload-lc-wasm.sh` | build + gov-upload the light client, patch hermes config |
| `stellaribc import-keys` | `hermes-keys.sh` | import the relayer keys (= router admin key) |
| `stellaribc client create-cosmos [--cli]` | `f1-create-cosmos-client.sh` / `f1-create-client.sh` | F1.1 — Cosmos client on Stellar |
| `stellaribc client create-stellar [--force]` | `f1-create-stellar-client.sh` | F1.2 — Stellar client on Cosmos (08-wasm) |
| `stellaribc client list` | api `/stellar/clients` | list router clients |
| `stellaribc counterparty stellar` | `f1-register-counterparty-stellar.sh` | F1.3 (pending Task 3) |
| `stellaribc counterparty cosmos` | `f1-register-counterparty-cosmos.sh` | F1.4 (pending Task 3) |

## Typical first run

```sh
stellaribc doctor              # confirm docker/stellar/cargo + .env
stellaribc bootstrap           # F0: images, chains, contracts, wasm, keys
stellaribc status              # everything green?
stellaribc client create-cosmos    # F1.1
stellaribc client create-stellar   # F1.2
stellaribc client list
```

Configuration is read from `stellar-ibc/.env` (shell env wins, matching the
scripts). The `counterparty` commands run their script if present, otherwise
print what's still blocked — see [`docs/TASKS.md`](../../../docs/TASKS.md)
Task 3 and [`docs/component-integrations.md`](../../../docs/component-integrations.md).
