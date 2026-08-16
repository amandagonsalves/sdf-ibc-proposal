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

## The problem

Stellar settles payments in five seconds for a fraction of a cent, and it holds
real value doing it: roughly **$820M in USDC**, **over $2B in tokenized
real-world assets**, and **$5.5B in stablecoin payment volume in Q1 2026** alone.
Almost all of that value can only move within Stellar.

Getting it out today means going through a custodial bridge or a signing
committee: a permissioned group that watches one chain and vouches for what
happened on the other. That group becomes the real security of every transfer
through it, no matter how secure the two chains are. It is also the single most
exploited design in the industry, **$2.5B+ in losses**, and in 2022 roughly **70%
of all crypto theft**. The three largest (Ronin $625M, Poly $611M, Wormhole
$326M) were all the same failure: compromise the committee, take the funds.

So Stellar's connection to the rest of crypto is currently capped by the weakest
intermediary in the path, and for a network whose whole premise is that trust
should be named, scoped and revocable, that is the wrong shape.

## What we are building

**Stellar, connected to the IBC network, with verification, not trust.**

Interstellar implements **IBC v2 (Eureka)** for Stellar natively. Two pieces:

- **Soroban contracts** implementing the Interchain Standards, so Stellar can
  send and receive IBC packets as a first-class member of the protocol rather
  than through an adapter.
- **A Stellar light client compiled to wasm**, which other chains load in order
  to verify Stellar for themselves.

There is **no operator, committee or attestation service anywhere in the trust
path**. Each chain runs on-chain code that checks the other chain's consensus
signatures directly. The relayer that carries the bytes has no authority at all:
if it lies, the proofs fail; if it disappears, packets expire and funds return to
the sender.

## Why this is possible now

IBC was effectively closed to chains outside its original family for years. The
protocol was too heavy to implement elsewhere, and joining meant convincing
another chain to fork its own binary in order to accept you.

Two changes removed that. **IBC v2** collapsed the old multi-step connection and
channel handshakes into a single counterparty registration and cut the provable
state from eight paths to three, which is what makes the protocol small enough to
run inside Soroban's storage and gas budget. And the **`08-wasm`** client module
lets a host chain load a light client as data, with no fork and no coordinated
upgrade on its side.

This is the first moment Stellar has been technically able to join IBC as a full
member rather than as a wrapped asset on someone else's bridge, and one of the
first opportunities for any chain outside the protocol's original family to do
so.

## What it unlocks

**Reach, immediately and then automatically.** IBC connects more than **115
chains** today. It has been in production since April 2021 with **no
protocol-level exploit**, secures roughly **$40B a year**, and settles a median
transfer in about **19 seconds**. Connecting Stellar once reaches all of it, and
every chain that joins IBC afterwards becomes reachable from Stellar at close to
zero marginal cost, because the protocol layer, the light client and the relayer
are all shared. A bridge connects two chains. A protocol implementation connects
Stellar to a graph, including the parts of it that do not exist yet.

**A two-way asset street.** Stellar's regulated stablecoins, tokenized RWAs and
non-USD fiat tokens become tradeable, lendable and usable as collateral
elsewhere without wrapping through several bridges first. In the other direction,
assets from those chains reach Stellar's anchor network, cash-out points and
remittance corridors. Stellar arrives as a *supplier* of asset classes the graph
is thin on, not just another consumer of USDC.

**A claim that survives scrutiny.** Every alternative route out of Stellar today
relocates the trusted party rather than removing it, which means the honest
version of their pitch is "trust our operator set." That claim cannot be checked
by the person relying on it. This one can, in three specific ways. The **evidence
is public**, because Stellar's consensus messages and ledger headers are
published in the history archives and anyone can fetch the same bytes the light
client consumes. The **verifier is open and runnable**, so a sceptic can point it
at a ledger and watch each check pass or fail rather than taking a diagram on
faith. And the **trusted set is enumerable**: there is exactly one assumption
that a contract cannot discharge, it is written down, and it has a mitigation and
an owner rather than a footnote.

**Proof that Soroban carries systems work.** A deterministic sparse Merkle tree
maintained on-chain, ICS-23 proof verification, and consensus verification
contracts are not application logic. Shipping them is an argument about the
platform, made in code.

**A path beyond asset transfer.** Token transfer is the first application on the
packet layer, not the ceiling, and IBC v2 makes that unusually cheap to exploit.
Under v1 an application was bound to a channel at handshake time, so every new
use case meant new plumbing. V2 carries the source and destination ports inside
the packet payload, so **a single client pairing already carries packets for any
number of applications**, including ones not yet written. Nothing about the trust
model changes when a second application appears; it rides the verification
already built. That opens **ICS-27 interchain accounts** (a contract elsewhere
controlling a Stellar account without a custodian), **ICS-31 cross-chain
queries** (a Soroban contract reading verified state from another chain), and
Soroban as a *callable destination* rather than only a source of assets. The
expensive part is paid once: the second and third product on this rail cost a
fraction of the first.

{: .note }
> **Why this is fundable as infrastructure.** Bespoke bridges scale O(*n²*),
> ~n²/2 pairwise integrations, each its own audit and operator set. IBC scales
> O(*n*): *n* light clients + 1 shared protocol + 1 generalized relayer. The
> Stellar light client is written once and loadable by any host chain, so the
> same investment that connects Stellar to its first counterparty also ships the
> Stellar half of every future pairing, because the protocol layer is shared.

## How this differs from a bridge

| Route | Who verifies a transfer | What you are trusting |
|---|---|---|
| **Interstellar (IBC)** | The destination chain's on-chain light client of Stellar | Stellar's consensus and the destination chain's consensus. **Nothing else** |
| **Committee bridges** | A validator set, DVN quorum or oracle network in the middle | An honest majority of a third party that neither chain controls |

That difference is the product. It is why a compromised relayer costs a delay
rather than the float, and why adding the next chain does not mean auditing
another operator set.

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
  timeout), and the inbound light client.
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
  the wasm light client, creates clients, registers counterparties, runs the
  services, and ships the consensus verifier.

Provable state is a deterministic fixed-depth-64 **Sparse Merkle Tree** whose
root is the consensus root counterparty light clients verify against, with proofs
serialized as ICS-23 `MerkleProof`s. The router recomputes and publishes that
root on every provable write, so it is correct by construction rather than
asserted by an off-chain service.

## The security model

One rule shapes the whole system: **no component holds bridge funds, and no
component vouches for events off-chain.** The relayer, the gateway and the api
are transport. They can slow a packet down or refuse to carry it. They cannot
forge one. Neither off-chain service holds a signing key at all.

The hard engineering is proving Stellar's consensus to another chain, and it
takes **two separate proofs that are easy to mistake for one**:

1. **A quorum of Stellar nodes agreed on a value for this ledger.** Stellar has
   no global validator list and no stake weighting, so there is no percentage to
   count to. The client has to evaluate *Stellar's own definition of agreement*
   against a configuration it trusts.
2. **What they agreed on is this ledger, and this ledger commits to the data
   being proved.** SCP commits to a transaction-set hash, a close time and
   upgrades. It does not commit to the state root, the transaction results, or
   even the previous ledger's hash.

Valid signatures on their own say nothing about which ledger or which state root
arrived alongside them. A design that treats those two proofs as one ends up
doing real cryptography and proving nothing useful. **Into Stellar** the problem
is easier and well-trodden: the counterparty has one globally agreed validator
set with voting power attached, and its header commits to the application state
root directly, so verifying the header yields the root.

The full chain (the eight checks in order, what each one establishes, what it was
verified against, and the one assumption that cannot be discharged on-chain) is
set out in [Architecture § 4](architecture.html).

## Evidence it works

The verification chain is implemented on-chain, and it has been run against live
**mainnet and testnet** data by a checker shipped in the CLI
(`interstellar verify`) that calls the contract's own code.

| What was checked | Result |
|---|---|
| Signatures, quorum evaluation, agreement, ledger binding | Confirmed on real archived ledgers: 21 signers under the nested mainnet configuration, 3 under a flat testnet one |
| The ledger commits to its transaction results | Reconstructed for **64 of 64** mainnet ledgers in a checkpoint |
| The result commits to the contract state root | Reconstructed for **40 of 40** Soroban invocations |
| Each check rejects bad input rather than passing it | **18-case negative suite**, one mutation per case, all fail closed |

None of this required a change to `stellar-core`. Stellar already publishes
everything the verification needs, which means this works against the network as
it is, with no protocol change to negotiate and no dependency on the core team's
roadmap. Full detail, including the shape of the codebase, is on
[Implementation & Evidence](implementation.html).

## Status by Interchain Standard

Progress is tracked against the Interchain Standards the stack implements, not
against ad-hoc implementation phases.

| ICS standard | What it covers here | State |
|---|---|---|
| **ICS-26. Routing** | `ibc-router` dispatch + IBC v2 counterparty registration (both sides) | done |
| **ICS-24. Host paths** | commitment / receipt / ack paths in the provable SMT store | done, byte-exact against the reference implementation |
| **ICS-02. Clients** | the inbound client on Stellar, and the Stellar client hosted via `08-wasm` | both implemented on-chain; lifecycle hardening in progress on the inbound client |
| **ICS-23. Commitments** | membership / non-membership `MerkleProof`s over the SMT | done |
| **ICS-04. Packets** | `send` / `recv` / `acknowledge` / `timeout` | done |
| **ICS-20. Token transfer** | escrow → relay → credit (`FungibleTokenPacketData`), over the Stellar Asset Contract token interface | done; denom-trace path prefixing not implemented |

{: .warning }
> **Early, under active development, a test implementation, not
> production-ready.** Packet flows run end to end on a devnet. The Stellar
> consensus verification described above is implemented on-chain, wired through
> the off-chain services and the relayer, and validated link by link against
> **live mainnet data** by a reference checker, including a negative suite
> showing each check fails closed. The relayer pins its trust root against a
> shipped constant rather than accepting one over the wire. Ahead of production:
> the on-chain verification cost budget, client-lifecycle hardening, and a
> third-party security audit before any value moves.

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
(each tagged with its ICS standards), see the
[Architecture](architecture.html) page. For the objections we consider fair and
how they are answered, see [Questions & Objections](questions.html).

*Stellar and IBC network figures are July 2026, from DeFiLlama, Map of Zones, the
IBC Protocol documentation and SDF's Q1 2026 report.*
