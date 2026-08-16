---
title: Implementation & Evidence
layout: default
nav_order: 6
description: >-
  What is built, the shape of the codebase, and the results of running the
  Stellar consensus verification against live mainnet and testnet data.
---

# Implementation & Evidence
{: .no_toc }

What exists today, how large it is, and what happened when the consensus
verification was run against live network data. The
[Architecture](architecture.html) page describes how the system is supposed to
work; this page is the evidence that it does.

{: .note }
> The implementation is currently in a private repository and will be
> open-sourced once it stabilizes. Figures below are from the working tree, and
> artifact sizes are from a clean release build.

## Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 1. What is built

### On Stellar

**`ibc-router`** is the IBC v2 core: it registers client types and
counterparties, dispatches `send` / `recv` / `acknowledge` / `timeout`, and owns
the provable store. It also maintains the Sparse Merkle Tree root on-chain,
recomputing and publishing it once per invocation, so a multi-write entrypoint
still emits exactly one root event.

**`ibc-transfer`** is the ICS-20 application, moving value over the SEP-41 /
Stellar Asset Contract token interface, with **`ibc-voucher-token`** as the token
it deploys for inbound denominations. The **inbound light client** verifies the
counterparty's consensus. Two development-only clients exist for testing and are
excluded from real deployments.

### On the host chain

**`light-client-wasm`** is the Stellar light client, loaded through the
`08-wasm` module. It implements the whole chain of custody described in
[Architecture § 4](architecture.html), plus the quorum-configuration history,
misbehaviour handling and expiry. It is the only Stellar-specific code the host
chain runs, and it runs **as data** rather than as a fork of the host's binary.

### `interstellar-api`

The api owns the **only Soroban RPC connection in the system** and holds no
signing key. Fourteen HTTP routes, in three groups:

- **Chain reads**: the latest ledger, a ledger by sequence, contract events,
  registered clients and their state, token balances.
- **Archive reads**: the SCP messages, ledger header and transaction set for a
  given ledger. This is where the consensus evidence comes from.
- **Transaction handling**: builds unsigned envelopes and submits pre-signed
  ones, so the service can construct any router call without ever being able to
  authorise one.

An OpenAPI document is generated from the handlers, and a test asserts that every
routed endpoint appears in it, so the published surface cannot silently drift
from the implemented one.

### `interstellar-gateway`

The gateway is the only component the relayer talks to, and it is built so that
compromising it cannot move value: **no signing key, no chain connection of its
own**, every call fulfilled through the api. It exposes twenty gRPC methods
across a query service and a message service, plus four more on the proof API.

Its substantive work is assembling evidence. For a given ledger it produces the
ICS-23 proof of the packet commitment, and the header bundle the light client
needs: the `EXTERNALIZE` messages for that ledger and its successor, the
quorum-set preimages those messages committed to, the transaction set that binds
the two ledgers, and the state-root proof walked out of the ledger's transaction
results down to the router's own event. It also serves the quorum configuration
and the router identity a client needs at creation, which the relayer treats as a
**convenience rather than an authority**.

### The relayer today

The link runs on a fork of Hermes. The Stellar side is **23 files and roughly
7,900 lines across 94 commits** on the integration branch, and it consists of
four pieces: a chain endpoint implementing Hermes's interface for Stellar, the
Stellar client and consensus types, a packet worker, and the configuration to
wire them together.

The packet worker is the part with no upstream equivalent. IBC v2 has no
channels, so the worker is **client-paired rather than channel-paired**, and it
carries a proof source, a client updater and a submitter for each direction
independently. It also performs the check that keeps the trust model intact:
before a quorum configuration is used, its `sha256` is compared against a
constant compiled into the relayer, and a mismatch is a **startup failure** rather
than a silent substitution.

Everything Stellar-specific living inside a fork has one structural cost. Every
upstream Hermes release has to be merged, and the surface that has to be kept
compatible is the whole chain-endpoint interface rather than a narrow contract.

### The relayer next

The migration under way is to the **Cosmos IBC v2 relayer**, and the reason is
architectural rather than a preference between codebases. That relayer does not
embed chain-specific proof logic. It obtains proofs from a separate **proof API**
over gRPC, which means the Stellar-specific work moves behind a service boundary
this project already owns, and the fork is retired along with the burden of
tracking upstream.

The proof API is already implemented in the gateway on the same port as its other
services, so **the proof half of the integration needs no fork**. The relayer runs
alongside Postgres under its own compose profile, with both chains configured and
the routing authored: client identifiers per direction, per-direction batch
sizes, and an acknowledgement policy that relays success acknowledgements rather
than dropping them, because ICS-20 escrow only settles once the success
acknowledgement gets home. Batching is deliberately pinned to one packet in the
Stellar direction, since Soroban allows a single host-function invocation per
transaction and the router's receive entrypoint takes one packet.

{: .note }
> **What the migration still needs** is a Stellar chain type in the relayer
> itself. Its chain-type abstraction currently covers Cosmos, EVM and SVM, and a
> configuration entry of any other type is skipped when the bridge clients are
> built, so **the routing is correct and inert until that lands**. The work is
> transaction construction and submission as Soroban invocations, signing through
> either a remote signer backed by the api or a local key, and a finality rule,
> which for Stellar is simply that a ledger is final once SCP externalizes its
> value.

### The CLI and the verifier

`interstellar` is a single binary that operates the whole stack: twelve top-level
commands covering installation and health checks, building the two wasm targets,
bringing services up and down, deploying contracts, creating clients and
registering counterparties, originating transfers, driving the relayer, and
running the live test flows. Configuration lives in one `.env` file that the
deploy commands write back to, so a deployment's contract addresses and client
identifiers are recorded where every other command reads them.

It also ships `verify`, the consensus checker used throughout this page. **The
same verifier exists as a standalone crate** of about 3,300 lines, so it can be
run and audited without the rest of the project. Both share the same step
modules, and a script proves the two copies identical rather than leaving them to
drift.

---

## 2. The shape of the codebase

| | Amount |
|---|---|
| Commits in the main repository | **529**, between 13 May and 3 August 2026 |
| Rust in the main repository | **29,730 lines** across 186 files |
| Stellar code in the relayer fork | **7,916 lines** across 23 files, on 94 commits |
| The standalone verifier | **3,304 lines** across 19 files |
| Tests passing | **316**, across the two cargo workspaces, none failing |

Where that Rust sits:

| Area | Lines | Files |
|---|---|---|
| Orchestrator CLI and the verifier | 8,291 | 61 |
| Soroban contracts | 7,147 | 46 |
| Stellar light client for host chains | 5,017 | 22 |
| Shared core library | 3,548 | 20 |
| `interstellar-api` | 2,104 | 17 |
| Cross-crate integration tests | 1,932 | 9 |
| `interstellar-gateway` | 1,691 | 11 |

The compiled artifacts, which is what actually gets deployed:

| Artifact | Size | Where it runs |
|---|---|---|
| Stellar light client | 469 KB | The host chain, loaded via `08-wasm` |
| `ibc-router` | 27 KB | Soroban |
| `ibc-transfer` | 18 KB | Soroban |
| `ibc-voucher-token` | 18 KB | Soroban |
| Inbound light client | 18 KB | Soroban |

The Soroban contracts are small **because IBC v2 is small**: three provable
paths, no connection or channel objects, and no handshake state machine. The
host-chain artifact is larger because it carries the XDR decoders, the quorum
evaluation and the full verification chain, and it is a release build before any
size-optimisation pass.

---

## 3. Mainnet verification results

The checker (`interstellar verify --ledger <n>`) fetches real archived consensus
messages, quorum sets and ledger headers from Stellar's public history archives
and runs every check the contract must run, **using the contract's own
primitives**. It names the step that fails rather than reporting a single
pass/fail.

| Checked | Result |
|---|---|
| Signatures, quorum evaluation, agreement, ledger binding, next-slot binding | Confirmed on live **mainnet and testnet** ledgers: 21 signers under the nested mainnet configuration, 3 under a flat testnet one |
| The ledger commits to its transaction results | Reconstructed for **64 of 64** mainnet ledgers in a checkpoint |
| The result commits to the contract state root | Reconstructed for **40 of 40** Soroban invocations across eight consecutive testnet ledgers |

The mainnet quorum structure observed at ledger **63907880** in August 2026 was
*5 of 7 organisations, each satisfied by 2 of its 3 validators*.

{: .note }
> **No change to `stellar-core` was required.** Stellar already publishes
> everything the verification consumes, which means this works against the
> network as it is: no protocol change to negotiate, and no dependency on the
> core team's roadmap.

---

## 4. Test vectors

A verified run can be frozen with `--export` and replayed with `--fixture`, which
gives **deterministic regression tests over real consensus data with no network
access**. A fixture carries the envelopes, the quorum sets, the ledger header, the
next-slot evidence and the expected ledger hash.

The tool also cross-checks its own XDR reader against the contract's. Both have
to agree on exactly where a signed statement starts and ends, so a disagreement
shows up as an explicit error instead of an unexplained signature failure.

---

## 5. Negative tests

An **18-case suite** (`--negative`) applies exactly one mutation to verified input
per case and asserts that the expected check rejects it. A case that is accepted,
**or that is rejected by the wrong check**, fails the suite.

| Check | Cases |
|---|---|
| Envelope signatures | tampered signature; wrong network id; tampered statement (ballot counter) |
| Statement validation | non-`EXTERNALIZE` statement; wrong slot; duplicate signer |
| Quorum-set authentication | missing preimage; preimage that hashes to something else; malformed nested set; zero threshold |
| Quorum check | signers outside the configured trust root; a single signer is not a quorum; no envelopes at all |
| Value agreement | conflicting committed values |
| Ledger binding | value does not match `header.scpValue`; header for another ledger; truncated header XDR; wrong recorded ledger hash |

This is the property that matters most. A verifier that accepts correct input may
not be doing anything at all; one that demonstrably refuses tampered signatures,
wrong network identifiers, replayed signers, forged quorum sets and mismatched
headers is performing verification.

---

## 6. End-to-end proof

Packet flows run end to end on a devnet: **send, receive, acknowledge and
timeout**, with commitments, receipts and acknowledgements exercised against a
live Soroban testnet and a local host chain. The acknowledgement check fails the
run when no acknowledgement appears, so a silently broken leg cannot pass as
success.

{: .warning }
> **What is still ahead of a production deployment.** Measuring the on-chain cost
> of verifying the consensus bundle and the state-root proof against a host
> chain's limits (see [Architecture § 8](architecture.html#8-performance)),
> finishing the client-lifecycle hardening on the inbound Soroban client, and a
> third-party security audit before any value moves. The quorum configuration the
> client trusts is a deliberate, reviewable setting and needs a named owner,
> which is an organisational decision rather than an engineering one.
