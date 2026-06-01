# `stellaribc` — Stellar↔Cosmos IBC orchestrator CLI

A front door to the `ci/flows/*.sh` scripts and services, grouped by
component. One binary instead of remembering script names and skip flags. It
discovers the repo root (walking up for `ci/flows/_env.sh`, or
`STELLAR_IBC_ROOT`), loads `.env` declaratively, probes service health
natively, and delegates the actual work to the canonical flow scripts (one
source of truth).

This crate lives at the repo root (`stellar-ibc/cli/`) and is a member of the
workspace — it shares the workspace `Cargo.lock` and `[workspace.dependencies]`.

## Install / run

```sh
# run in-place (from the repo root)
cargo run -p stellar-ibc-cli -- <command>

# install the `stellaribc` binary (any of these)
cargo run -p stellar-ibc-cli -- install   # self-install
make -C ci install
cargo install --path cli
```

## Source layout

```
src/
  main.rs                 clap command tree + dispatch
  logger.rs               ui logger (banner/step/ok/warn/fail/detail/hint/status_line)
  config.rs repo.rs run.rs probe.rs shared.rs    support
  ops/        install, doctor, status, stack (up/down), bootstrap
  clients/    cosmos (F1.1), stellar (F1.2), counterparty (F1.3/F1.4), list
  hermes/     image, keys, start
  gateway/    image, query
  api/        image
  contracts/  deploy, wasm (upload light client)
  tx/         clients, msg, query   (low-level surface)
```

## Commands

| Command | Wraps | What it does |
|---|---|---|
| `stellaribc install` | `cargo install` | install the binary to the cargo bin dir |
| `stellaribc doctor` | — | check toolchain, `.env`, config, service health |
| `stellaribc status` | — | probe chains/services, show contracts + created clients |
| `stellaribc up [--cosmos\|--stellar]` | `docker compose` | start osmosis + api + gateway |
| `stellaribc down [--volumes]` | `docker compose` | stop the stack |
| `stellaribc bootstrap` (alias `f0`) | `f0-bootstrap.sh` | full F0; `--skip-images/-contracts/-wasm/-keys`, `--force-redeploy` |
| `stellaribc clients cosmos [--cli]` | `f1-create-cosmos-client.sh` / `f1-create-client.sh` | F1.1 — Cosmos client on Stellar |
| `stellaribc clients stellar [--force]` | `f1-create-stellar-client.sh` | F1.2 — Stellar client on Cosmos (08-wasm) |
| `stellaribc clients counterparty <stellar\|cosmos>` | `f1-register-counterparty-*.sh` | F1.3 / F1.4 (pending Task 3) |
| `stellaribc clients list` | api `/stellar/clients` | list router clients |
| `stellaribc hermes build-image` | `docker build` | build the hermes image from the `hermes-relayer` repo (native) |
| `stellaribc hermes push-image [--rebuild]` | `docker push` | push the hermes image (login from `.env` creds) |
| `stellaribc hermes start [--rebuild]` | `docker compose up` | start the hermes relayer container |
| `stellaribc hermes stop` | `docker compose stop` | stop the hermes relayer container |
| `stellaribc hermes restart [--rebuild]` | `docker compose` | restart (recreate on `--rebuild`) |
| `stellaribc hermes keys-import` | `docker compose run` | import the relayer keys natively (= router admin key) |
| `stellaribc gateway build-image` | `docker build` | build the gateway image (native) |
| `stellaribc gateway push-image [--rebuild]` | `docker push` | push the gateway image (login from `.env` creds) |
| `stellaribc gateway start [--rebuild]` | `docker compose up` | start the gateway container |
| `stellaribc gateway stop` | `docker compose stop` | stop the gateway container |
| `stellaribc gateway restart [--rebuild]` | `docker compose` | restart (recreate on `--rebuild`) |
| `stellaribc gateway query` | — | direct gRPC reads (pending) |
| `stellaribc api build-image` | `docker build` | build the api image (native) |
| `stellaribc api push-image [--rebuild]` | `docker push` | push the api image (login from `.env` creds) |
| `stellaribc api start [--rebuild]` | `docker compose up` | start the api container |
| `stellaribc api stop` | `docker compose stop` | stop the api container |
| `stellaribc api restart [--rebuild]` | `docker compose` | restart (recreate on `--rebuild`) |
| `stellaribc contracts build` | `stellar contract build` | build all Soroban contracts to wasm (native) |
| `stellaribc contracts upload --wasm <p>` | `stellar contract upload` | upload a wasm, print the hash (native) |
| `stellaribc contracts deploy --wasm <p> -- <ctor...>` | `stellar contract deploy` | deploy a wasm, print the id (native) |
| `stellaribc contracts invoke --id <c> -- <fn> <args...>` | `stellar contract invoke` | invoke a contract function (native) |
| `stellaribc contracts deploy-all [--force]` | `upload-and-deploy-contracts.sh` | full orchestration: build/upload/deploy + wire router + write `.env` |
| `stellaribc contracts upload-wasm` | `upload-lc-wasm.sh` | gov-upload the light client, patch hermes config |
| `stellaribc tx clients <create\|update>` | — | low-level client txs (pending Task 3) |
| `stellaribc tx msg <register-counterparty\|recv\|ack\|timeout>` | — | packet/counterparty msgs (pending Task 3/5) |
| `stellaribc tx query <commitment\|receipt\|ack\|header>` | — | provable-path queries (pending) |

## Typical first run

```sh
stellaribc doctor               # confirm docker/stellar/cargo + .env
stellaribc bootstrap            # F0: images, chains, contracts, wasm, keys
stellaribc status               # everything green?
stellaribc clients cosmos       # F1.1
stellaribc clients stellar      # F1.2
stellaribc clients list
```

Configuration is read from `stellar-ibc/.env` (shell env wins, matching the
scripts). Commands not yet wired print a clear "not wired yet" notice — see
[`docs/TASKS.md`](../../docs/TASKS.md) Task 3 and
[`docs/component-integrations.md`](../../docs/component-integrations.md).
