---
title: Home
layout: home
description: >-
  IBC v2 (Eureka) for Stellar, on-chain light-client verification of consensus,
  with no bridge operator, custodian or attestation service in the trust path.
permalink: /
---

{: .note }
> **Source availability.** The implementation is currently in a private
> repository while it is under active development. It will be **open-sourced**
> once it stabilizes. This site is the public documentation of the project's
> design, rationale, and roadmap in the meantime.

## What it is

**Stellar, connected to the IBC network, with verification, not trust.**

Moving value between chains normally means trusting somebody: a bridge operator,
a multisig, an attestation service. Whoever that is becomes the weakest point.
IBC removes them. Each chain runs a **light client** of the other (on-chain code
that checks the other chain's consensus signatures itself) and every packet is
accompanied by a Merkle proof verified against a header that client has already
verified. The relayer carries bytes and holds no authority: if it lies, proofs
fail; if it disappears, packets time out and funds are refunded.

This project implements that for **Stellar** (Soroban smart contracts, SCP
consensus) using **IBC v2 (Eureka)**, the streamlined protocol that drops the
v1 connection and channel handshakes, keeping only the packet lifecycle.

The deliverable is not a route to one destination. It is **membership in a
protocol**: IBC connects **115+ chains** today, and once Stellar speaks it, every
one of them is reachable without new bridge code, a new operator set, or a new
security model to evaluate. Each chain that joins IBC afterwards becomes
reachable from Stellar at near-zero marginal cost.

The defining property is that **no component holds bridge funds or attests to
events off-chain.** Cross-chain authenticity is verified by on-chain light
clients; the relayer, gateway, and api are untrusted transport. A malicious
relayer can stall or censor, but cannot mint, steal, or forge a transfer, the
security of a packet equals the security of the two underlying chains.

{: .note }
> **Why this is fundable as infrastructure.** Bespoke bridges scale O(*n²*),
> ~n²/2 pairwise integrations, each its own audit and operator set. IBC scales
> O(*n*): *n* light clients + 1 shared protocol + 1 generalized relayer. The
> Stellar light client is written once and loadable by any host chain, so the
> same investment that connects Stellar to its first counterparty also ships the
> Stellar half of every future pairing, because the protocol layer is shared.

## How it works

Authenticity is checked by an **on-chain light client of the source chain,
running inside the destination chain**. A packet sent on one chain is committed
to that chain's provable state; the relayer carries the packet plus a Merkle
proof to the other chain, whose light client verifies the proof against a header
it has already accepted.

The pieces:

- **Soroban contracts**, the `ibc-router` (IBC v2 core: client/counterparty
  registration, `send` / `recv` / `ack` / `timeout`, and the provable
  commitment/receipt/ack store, whose root is computed **on-chain**), the
  `ibc-transfer` ICS-20 application (escrow on send, credit on receive, refund on
  timeout), and the inbound light clients.
- **`light-client-wasm`**, the Stellar light client compiled to wasm and
  deployed on the counterparty via `08-wasm`; verifies signed SCP `EXTERNALIZE`
  statements, evaluates the quorum, binds the agreed value to a Stellar ledger,
  and binds that ledger to the state root proofs are checked against.
- **`interstellar-gateway`**, the keyless gRPC service the relayer talks to;
  serves proofs, events, and the consensus evidence the light client verifies. It
  also serves the proof API the IBC v2 relayer expects.
- **`interstellar-api`**, the HTTP service that owns the Soroban RPC connection
  and no signing key, building unsigned transactions and submitting ones signed
  by their originator.
- **Relayer**, the link runs today on a fork of **Hermes** carrying a Stellar
  chain endpoint and a channel-less v2 packet worker. It is being migrated to the
  **IBC v2 relayer**, which takes proofs from a separate proof API rather than
  embedding chain-specific proof logic.
- **`interstellar` CLI**, the orchestrator that deploys the contracts, uploads
  the wasm light client, creates clients, registers counterparties, and runs the
  services.

Provable state is a deterministic fixed-depth-64 **Sparse Merkle Tree** whose
root is the consensus root counterparty light clients verify against, with proofs
serialized as ICS-23 `MerkleProof`s. The router recomputes and publishes that
root on every provable write, so it is correct by construction rather than
asserted by an off-chain service.

## The security model

Everything rests on one question, asked in **each** direction: *did the other
chain's validators really agree on this?* Two light clients answer it, and they
are genuinely different problems.

**Into Stellar** is well-trodden. The counterparty has one globally agreed
validator set with voting power attached, so "validators holding more than
two-thirds of the power signed this block" is a well-defined statement, and its
header commits to the application state root directly. The Soroban client checks
exactly that.

**Out of Stellar** has no equivalent on either count. Stellar uses *Federated
Byzantine Agreement*: there is no single configured validator set and no stake
weighting. Each node chooses which sets of peers it will listen to, and agreement
is defined relative to those choices, so there is **no threshold to count to**.
The client instead evaluates whether a specific set of signers forms a **quorum**
under the configuration it trusts for that ledger, using Stellar's own recursive
rule. It then has to do something the other direction gets almost for free: tie
what was agreed to an actual ledger, and that ledger to the state root the proofs
are checked against.

Those are two separate claims, and both are proved explicitly. A quorum of valid
signatures establishes that consensus happened; on its own it says nothing about
which ledger or which state root travelled alongside it.

The full chain (the checks in order, what each one establishes, what it was
verified against, and the one assumption that cannot be discharged on-chain) is
set out in [Architecture § 4](architecture.html).

## Status by Interchain Standard

Progress is tracked against the Interchain Standards the stack implements, not
against ad-hoc implementation phases.

| ICS standard | What it covers here | State |
|---|---|---|
| **ICS-26. Routing** | `ibc-router` dispatch + IBC v2 counterparty registration (both sides) | done |
| **ICS-24. Host paths** | commitment / receipt / ack paths in the provable SMT store | done, byte-exact against the reference implementation |
| **ICS-02. Clients** | the inbound client on Stellar, and the Stellar client hosted via `08-wasm` | both implemented on-chain |
| **ICS-23. Commitments** | membership / non-membership `MerkleProof`s over the SMT | done |
| **ICS-04. Packets** | `send` / `recv` / `acknowledge` / `timeout` | done |
| **ICS-20. Token transfer** | escrow → relay → credit (`FungibleTokenPacketData`), over the Stellar Asset Contract token interface | done |

{: .warning }
> **Early, under active development, a test implementation, not
> production-ready.** Packet flows run end to end on a devnet. The Stellar
> consensus verification described above is implemented on-chain, wired through
> the off-chain services and the relayer, and validated link by link against
> **live mainnet data** by a reference checker, including a negative suite
> showing each check fails closed. The relayer pins its trust root against a
> shipped constant rather than accepting one over the wire. Broader test
> coverage, a verification cost budget, and a security review are still ahead.

## A transfer in ICS terms

The flows map directly onto the Interchain Standards (no v1 connection/channel
handshake, IBC v2 keeps only the packet lifecycle):

- **Setup**, `RegisterCounterparty` per side (**ICS-26**), binding each client to
  its counterparty id and commitment prefix (**ICS-24**).
- **Outbound**, `ibc-transfer` escrows the asset and builds the
  `FungibleTokenPacketData` (**ICS-20** `OnSendPacket`); `ibc-router.send_packet`
  writes the commitment (**ICS-04** / **ICS-24**); the relayer proves it
  (**ICS-23**) and the counterparty's Stellar light client verifies the consensus
  chain (**ICS-02** `VerifyClientMessage` → `UpdateState`) and the commitment
  (**ICS-23** `VerifyMembership`) on-chain, then credits the receiver (**ICS-20**
  `OnRecvPacket`).
- **Ack back**, the success acknowledgement is proven (**ICS-23**) and relayed to
  `ibc-router.acknowledge_packet` (**ICS-04**), which verifies it via the inbound
  light client, clears the commitment, and settles the escrow (**ICS-20**
  `OnAcknowledgementPacket`). Timeouts refund via an **ICS-23** non-membership
  proof.

For the full trust model, component breakdown, and per-flow sequence diagrams
(each tagged with its ICS standards), see the [Architecture](architecture.html) page.
