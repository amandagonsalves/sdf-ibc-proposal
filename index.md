---
title: Home
layout: home
description: >-
  IBC v2 (Eureka) for Stellar — on-chain light-client verification of consensus,
  with no bridge operator, custodian or attestation service in the trust path.
permalink: /
---

{: .note }
> **Source availability.** The implementation is currently in a private
> repository while it is under active development. It will be **open-sourced**
> once it stabilizes. This site is the public documentation of the project's
> design, rationale, and roadmap in the meantime.

## What it is

**Stellar, connected to the IBC network — with verification, not trust.**

Moving value between chains normally means trusting somebody: a bridge operator,
a multisig, an attestation service. Whoever that is becomes the weakest point.
IBC removes them. Each chain runs a **light client** of the other — on-chain code
that checks the other chain's consensus signatures itself — and every packet is
accompanied by a Merkle proof verified against a header that client has already
verified. The relayer carries bytes and holds no authority: if it lies, proofs
fail; if it disappears, packets time out and funds are refunded.

This project implements that for **Stellar** (Soroban smart contracts, SCP
consensus) using **IBC v2 (Eureka)** — the streamlined protocol that drops the
v1 connection and channel handshakes, keeping only the packet lifecycle. The
first counterparty is a Cosmos chain (ibc-go v10+ with the `08-wasm`
light-client module); the same machinery extends to Cardano and beyond.

The defining property is that **no component holds bridge funds or attests to
events off-chain.** Cross-chain authenticity is verified by on-chain light
clients; the relayer, gateway, and api are untrusted transport. A malicious
relayer can stall or censor, but cannot mint, steal, or forge a transfer — the
security of a packet equals the security of the two underlying chains.

It ships as reusable **infrastructure, not a point bridge**: the marginal cost of
the next chain is one light client plus one relayer chain-endpoint, so the same
stack reaches Cosmos today and Cardano (and multi-hop routes) next.

{: .note }
> **Why this is fundable as infrastructure.** Bespoke bridges scale O(*n²*) —
> ~n²/2 pairwise integrations, each its own audit and operator set. IBC scales
> O(*n*): *n* light clients + 1 shared protocol + 1 generalized relayer. The same
> dollar that ships a Stellar↔Cosmos bridge ships the Stellar half of every
> future Stellar↔non-Cosmos pair, because the protocol layer is shared.

## How it works

Authenticity is checked by an **on-chain light client of the source chain,
running inside the destination chain**. A packet sent on one chain is committed
to that chain's provable state; the relayer carries the packet plus a Merkle
proof to the other chain, whose light client verifies the proof against a header
it has already accepted.

The pieces:

- **Soroban contracts** — the `ibc-router` (IBC v2 core: client/counterparty
  registration, `send` / `recv` / `ack` / `timeout`, and the provable
  commitment/receipt/ack store), the `ibc-transfer` ICS-20 application — escrow
  and mint over the **Stellar Asset Contract (SAC)** token interface, so native
  XLM and issued assets (USDC, EURC) move by their canonical SAC address — and the
  on-chain light clients (`tendermint`, `attestation`, `mock`).
- **`light-client-wasm`** — the Stellar light client compiled to wasm and
  deployed on the counterparty via `08-wasm`; verifies signed SCP `EXTERNALIZE`
  statements, evaluates the quorum, binds the agreed value to a Stellar ledger,
  and checks ICS-23 proofs against the Stellar state root.
- **`interstellar-gateway`** — the keyless gRPC service the relayer talks to;
  tracks the state root and produces proofs. It also serves the proof API the
  Cosmos IBC v2 relayer expects.
- **`interstellar-api`** — the HTTP service that owns the Soroban RPC connection
  and the signing key, building and submitting transactions on the gateway's
  behalf.
- **Relayer** — the link runs today on a fork of **Hermes** carrying a
  `StellarChainEndpoint` and a channel-less v2 packet worker. It is being
  migrated to the **Cosmos IBC v2 relayer**, which takes proofs from a separate
  proof API rather than embedding chain-specific proof logic.
- **`interstellar` CLI** — the orchestrator that deploys the contracts, uploads
  the wasm light client, creates clients, registers counterparties, and runs the
  services.

Provable state is a deterministic fixed-depth-64 **Sparse Merkle Tree** whose
root is the consensus root counterparty light clients verify against, with proofs
serialized as ICS-23 `MerkleProof`s — a format shared with Cardano so the same
machinery serves both ecosystems.

## The security model

Everything rests on one question, asked in **each** direction: *did the other
chain's validators really agree on this?* Two light clients answer it, and they
are genuinely different problems.

**Cosmos → Stellar** is well-trodden. Tendermint has one globally agreed
validator set with voting power attached, so "validators holding more than
two-thirds of the power signed this block" is a well-defined statement, and the
Soroban client checks exactly that.

**Stellar → Cosmos** has no equivalent. Stellar uses *Federated Byzantine
Agreement*: there is no single configured validator set and no stake weighting.
Each node chooses which sets of peers it will listen to, and agreement is
defined relative to those choices — so there is **no threshold to count to**. The
client instead evaluates whether a specific set of signers forms a **quorum**
under the configuration it trusts for that ledger, using Stellar's own recursive
rule. It then has to do something a Cosmos client gets almost for free: tie what
was agreed to an actual ledger, and that ledger to the state root the proofs are
checked against.

Those are two separate claims, and both are proved explicitly. A quorum of valid
signatures establishes that consensus happened; on its own it says nothing about
which ledger or which state root travelled alongside it.

The full chain — eight checks, what each one establishes, what it was verified
against, and the one assumption that cannot be discharged on-chain — is set out
in [Architecture § 4](architecture.html).

## Status by Interchain Standard

Progress is tracked against the Interchain Standards the stack implements, not
against ad-hoc implementation phases. State as of the current devnet (live
Soroban testnet + an ibc-go v11 `simd` with `08-wasm`):

| ICS standard | What it covers here | State |
|---|---|---|
| **ICS-26 — Routing** | `ibc-router` dispatch + IBC v2 counterparty registration (both sides) | done |
| **ICS-24 — Host paths** | commitment / receipt / ack paths in the provable SMT store | done |
| **ICS-02 — Clients** | `07-tendermint` on Stellar, Stellar `08-wasm` on Cosmos — create / update / verify | done; `08-wasm` verified on-chain |
| **ICS-23 — Commitments** | membership / non-membership `MerkleProof`s over the SMT | membership verified on-chain; non-membership (timeout) implemented |
| **ICS-04 — Packets** | `send` + `recv` + `acknowledge` verified end-to-end (Stellar→Cosmos round trip closed on-chain); `timeout` implemented | done |
| **ICS-20 — Token transfer** | escrow → relay → mint over the Stellar Asset Contract (SAC) token interface (`FungibleTokenPacketData`) | Stellar→Cosmos proven on-chain; real-asset (SAC) escrow + reverse next |

{: .warning }
> **Early, under active development — a test implementation, not
> production-ready.** A single ICS-20 transfer Stellar→Cosmos has been relayed
> and **verified on-chain** by the `08-wasm` light client (SCP header +
> ICS-23/SMT commitment proof), after which Cosmos minted the IBC voucher with a
> success acknowledgement, and the acknowledgement back-leg was relayed to Stellar
> and verified by the `tendermint` light client — closing the ICS-04 round trip
> on-chain. The reverse direction (Cosmos→Stellar) is in progress; broader test
> coverage and a security review are still ahead.

## A transfer in ICS terms

The flows map directly onto the Interchain Standards (no v1 connection/channel
handshake — IBC v2 keeps only the packet lifecycle):

- **Setup** — `RegisterCounterparty` per side (**ICS-26**), binding each client to
  its counterparty id and commitment prefix (**ICS-24**).
- **Stellar → Cosmos** — `ibc-transfer` escrows the asset via its **SAC** token
  contract and builds the `FungibleTokenPacketData` (**ICS-20** `OnSendPacket`); `ibc-router.send_packet`
  writes the commitment (**ICS-04** / **ICS-24**); the relayer proves it
  (**ICS-23**) and the Cosmos `08-wasm` Stellar LC verifies the SCP header
  (**ICS-02** `VerifyClientMessage` → `UpdateState`) and the commitment
  (**ICS-23** `VerifyMembership`) on-chain, then mints the voucher (**ICS-20**
  `OnRecvPacket`).
- **Ack back** — the success ack (`{"result":"AQ=="}`) is proven (**ICS-23**) and
  relayed to `ibc-router.acknowledge_packet` (**ICS-04**), which verifies it via
  the `tendermint` LC, clears the commitment, and settles the escrow (**ICS-20**
  `OnAcknowledgementPacket`). Timeouts refund via an **ICS-23** non-membership
  proof.

For the full trust model, component breakdown, and per-flow sequence diagrams
(each tagged with its ICS standards), see the [Architecture](architecture.html) page.
