---
title: Architecture
layout: default
nav_order: 4
description: >-
  The trust model, components, data flows, and sequence diagrams of the
  Interstellar, the IBC v2 (Eureka) implementation for Stellar.
---

# Architecture
{: .no_toc }

How Interstellar is put together: the trust model, the
components and their responsibilities, the data flows that move a transfer across
chains, and how it all runs. Written for reviewers, integrators, and
contributors.

{: .note }
> The implementation is currently in a private repository and will be
> open-sourced once it stabilizes. The component and type names below
> (`ibc-router`, `StellarChainEndpoint`, `light-client-wasm`, …) describe that
> codebase.

## Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 1. System overview

The bridge connects **Stellar** (Soroban smart contracts, SCP consensus) to any
**IBC-enabled chain**. The first counterparty is a Cosmos chain (ibc-go v10+ with
the `08-wasm` light-client module; the local devnet runs an ibc-go v11 `simd`);
the same machinery extends to Cardano and beyond.

The defining property is that **no component holds bridge funds or attests to
events off-chain.** Verification happens inside on-chain light clients. The
relayer, gateway, and api are untrusted transport.

### Trust model

IBC is trust-minimized because packet authenticity is checked by an **on-chain
light client of the source chain, running inside the destination chain**:

- A packet sent on chain A is committed to A's provable state.
- The relayer carries the packet plus a Merkle proof to chain B.
- Chain B's light client of A verifies the proof against a header it has already
  accepted from A. If the proof is valid, the packet is genuine.

There is no validator committee, no multisig federation, no off-chain signer set
that can be compromised to forge a transfer. The security of a packet equals the
security of the two underlying chains — nothing weaker.

| Role | Holds funds? | Holds keys? | Trusted for correctness? |
|---|---|---|---|
| `ibc-router` + light clients (on-chain) | escrow only | — | **yes** — this is the verification root |
| `interstellar-api` | no | yes (relayer signing key) | no — only signs what the relayer asks |
| `interstellar-gateway` | no | no | no — pure transport/encoding |
| Hermes relayer | no | yes (its own fee key) | no — a wrong/missing relay cannot forge a packet, only delay it |

A malicious relayer can censor or stall, but cannot mint, steal, or forge — the
on-chain light client rejects any packet without a valid proof.

### Provable state (IBC v2)

IBC v2 (Eureka) keeps only **three** provable paths — the packet lifecycle —
versus eight in v1. This is decisive on Stellar, where Soroban storage is
rent-priced per byte:

| Value | Path bytes |
|---|---|
| Packet Commitment | `{sourceClientId} \|\| 0x01 \|\| be64(sequence)` |
| Packet Receipt | `{destClientId} \|\| 0x02 \|\| be64(sequence)` |
| Acknowledgement Commitment | `{destClientId} \|\| 0x03 \|\| be64(sequence)` |

These live in a **deterministic fixed-depth-64 binary Sparse Merkle Tree (SMT)**,
chosen to be **Cardano-compatible** so the same proof format serves both
ecosystems. The SMT root is the `ConsensusState.root` that counterparty light
clients verify against. Proofs are serialized as ICS-23 `MerkleProof`s:
membership proves a commitment exists (recv / ack); non-membership proves a
receipt is absent (timeout). Because client / connection / channel state is not
provable in v2, the gateway's client/consensus/next-sequence queries
intentionally return `Unimplemented`.

Encoding details that had to match ibc-go exactly: the packet-commitment timeout
is hashed **big-endian** (`sha256(be64(timeout))`); the counterparty merkle prefix
is **empty** on the Cosmos side and `ibc` on the Stellar side; a membership proof
embeds the **value hash**, so the light client compares `proof_value ==
sha256(value)`; ICS-20 data is JSON `FungibleTokenPacketData` with
`version = "ics20-1"`, `encoding = "application/json"`.

### Multi-chain extensibility

The investment is **infrastructure, not a point bridge**. With *n* IBC chains,
custom bridges cost about n²/2 pairwise integrations; IBC costs *n* light clients
+ 1 shared protocol + 1 generalized relayer. The marginal cost of the next chain
is **one light client + one chain endpoint**. Once two non-Cosmos chains both
speak IBC they talk directly — no Cosmos chain in the middle — which yields the
`stellar ↔ cardano` and multi-hop forwarding routes on the roadmap.

### Status by ICS standard

Progress is tracked against the Interchain Standards this stack implements, not
against ad-hoc implementation phases.

| ICS standard | Scope in this bridge | State |
|---|---|---|
| **ICS-26 — Routing Module** | `ibc-router` dispatch and IBC v2 counterparty registration on both sides | done |
| **ICS-24 — Host Requirements** | commitment / receipt / ack paths in the provable SMT store | done |
| **ICS-02 — Client Semantics** | `07-tendermint` LC on Stellar and the Stellar `08-wasm` LC on Cosmos — create / update / verify | done; `08-wasm` verification proven on-chain |
| **ICS-23 — Vector Commitments** | ICS-23 membership / non-membership `MerkleProof`s over the SMT | membership verified on-chain (recv); non-membership (timeout) implemented |
| **ICS-04 — Packet Semantics** | `send` + `recv` + `acknowledge` verified end-to-end (Stellar→Cosmos round trip closed on-chain); `timeout` implemented | done |
| **ICS-20 — Fungible Token Transfer** | escrow → relay → mint over the Stellar Asset Contract (SAC) token interface, `FungibleTokenPacketData` | Stellar→Cosmos proven on-chain; real-asset (SAC) escrow + reverse (Cosmos→Stellar) next |

IBC v2 (Eureka) has no connection or channel handshake, so the v1 ICS-03
(Connection) and the handshake half of ICS-04 (Channel) do not apply — packet
semantics survive in ICS-04, counterparty wiring moves to ICS-26.

---

## 2. Components

The system splits into four layers: the Stellar on-chain protocol, the Stellar
off-chain services, the relayer, and the counterparty-side light client plus
orchestration.

### a. Stellar on-chain layer

| Component | Responsibility |
|---|---|
| **`ibc-router`** | The IBC v2 core on Stellar. Registers client types and counterparties, dispatches `send` / `recv` / `ack` / `timeout`, and owns the provable commitment / receipt / ack store (the SMT). |
| **`ibc-transfer`** | ICS-20 application. Escrows on send, credits/mints on recv, refunds on timeout or failed ack, and settles its state on a successful ack — all over the **Stellar Asset Contract (SAC)** token interface, so native XLM and issued assets (USDC, EURC) move by their canonical SAC address rather than a bespoke token. Encodes and decodes `FungibleTokenPacketData`. |
| **light clients (`tendermint`, `attestation`, `mock`)** | Verify counterparty headers and membership proofs on Stellar. `tendermint` tracks a Cosmos chain; `mock` is always-accept for development; `attestation` is a roadmap variant. |
| **`stellar-ibc-core`** | Shared library underneath the contracts and services: the fixed-depth-64 SMT, the ICS-23 proof serializer, the IBC commitment paths, the client/consensus types and reverse codecs, the Soroban RPC client, and the HTTP `ApiClient`. |

### b. Stellar off-chain services

| Component | Responsibility |
|---|---|
| **`interstellar-gateway`** | The gRPC service the relayer talks to (`StellarGatewayQuery` + `StellarGatewayMsg`). Holds **no** Soroban connection and **no** key — every call is fulfilled through `ApiClient` against `interstellar-api`. Tracks the SMT root, produces ICS-23 proofs, and decodes Soroban router events into IBC-shaped attributes. |
| **`interstellar-api`** | The standalone HTTP service that owns the Soroban RPC connection and the signing key. Builds unsigned transactions, submits signed transactions, and exposes ledger / account / event reads plus Cosmos-side governance and bank helpers. Splitting it from the gateway means the key lives in exactly one place and the gateway stays a stateless protocol adapter. |

### c. Relayer — Hermes today, the Cosmos IBC v2 relayer next

The link runs end to end today on a fork of **Hermes** carrying Stellar support:
token transfers work in both directions, with commitments, receipts,
acknowledgements and timeouts exercised against a live Stellar testnet and a
Cosmos devnet.

| Component | Responsibility |
|---|---|
| **`StellarChainEndpoint`** | Implements Hermes's `ChainEndpoint` for Stellar: polls the gateway for events, builds IBC v2 messages, signs with the relayer key, submits via `SubmitSignedTx`, and queries clients / commitments / receipts / acks with proofs. |
| **`ics10-stellar` client types** | Stellar client / consensus state types and the v2 message encodings; unwraps the `08-wasm` envelope so the relayer can track the Stellar client on the counterparty like a native one. |
| **`stellar-packet` worker** | The custom v2 relay worker. IBC v2 has no channels, so it is **client-paired** rather than channel-paired; it drives recv, the ack-back leg, and timeout, carrying a proof source, client updater, and submitter for **each** direction. |

#### Migrating to the Cosmos IBC v2 relayer

That relayer is being migrated to the **Cosmos IBC v2 relayer**, for one
architectural reason above all: **it does not embed chain-specific proof logic.**
It obtains proofs from a separate **proof API** over gRPC, and is request-driven,
Postgres-backed, batches recv / ack / timeout independently, retries failed
submissions, tracks transaction costs, and supports remote signing.

The fit is good because the proof service already exists: `interstellar-gateway`
already implements that proof API alongside its own services, on the same gRPC
port, so **the proof half of the integration is already done and needs no fork**.
The relayer is wired up alongside Postgres with both chains configured and
routing authored, and the Cosmos side is operational.

What the migration adds is a Stellar chain type: transaction construction and
submission as Soroban `InvokeHostFunction` calls, signing (either via the
relayer's remote-signer interface backed by `interstellar-api`, or a local key),
and a finality rule — a ledger is final once SCP externalizes its value, with no
additional confirmations. One standing constraint: Soroban allows one
`InvokeHostFunction` per transaction and `ibc-router.recv_packet` takes a single
packet, so every Stellar-bound batch stays at one packet until the router grows a
batch entrypoint. Batching applies normally in the Cosmos direction.

Completing this retires the Hermes fork and the work of tracking it against
upstream.

### d. Counterparty light client & orchestration

| Component | Responsibility |
|---|---|
| **`light-client-wasm`** | The **Stellar** light client, compiled to wasm and deployed on the counterparty via `08-wasm`. Verifies signed SCP `EXTERNALIZE` statements, evaluates the quorum, binds the agreed value to a Stellar ledger, and checks ICS-23 proofs against the Stellar state root. `08-wasm` lets any ibc-go v10+ chain host it without forking its binary. |
| **`interstellar` CLI** | The orchestrator: deploys contracts, uploads the wasm light client, creates clients, registers counterparties, runs the services, and originates transfers. |

#### Light clients verify in both directions

A bridge needs each chain to verify the other, so there are two light clients:

- **Cosmos → Stellar** — the `tendermint` LC on Soroban (`07-tendermint`) accepts
  the Cosmos client/consensus state, verifies header updates, and checks ICS-23
  membership proofs against the stored consensus root.
- **Stellar → Cosmos** — `light-client-wasm` via `08-wasm`. Its model is
  deliberately **not** Tendermint-style. Stellar does not use a single globally
  configured validator set or stake-weighted voting, so there is no ">2/3 of the
  validators" to count. The client verifies signed SCP `EXTERNALIZE` statements
  for the slot — Ed25519 over
  `networkID ‖ xdr(ENVELOPE_TYPE_SCP) ‖ xdr(SCPStatement)` — and then evaluates
  whether the signers form a **quorum** under the quorum configuration it trusts
  for that ledger, using Stellar's own recursive rule rather than a threshold
  count. Section 4 walks through the full chain.

---

## 3. Data flows

Each flow is named with the Interchain Standards it exercises. The ICS operation
names are used directly: `send` / `recv` / `acknowledge` / `timeout` are ICS-04
(packet semantics); `OnSendPacket` / `OnRecvPacket` / `OnAcknowledgementPacket` /
`OnTimeoutPacket` are the ICS-26 routing callbacks into the ICS-20 application;
`VerifyClientMessage` / `UpdateState` are ICS-02 (client); `VerifyMembership` /
`VerifyNonMembership` are ICS-23 (commitments) over the ICS-24 host paths.

### Flow 1 — Counterparty registration · ICS-26 + ICS-24

IBC v2 replaces the v1 connection + channel handshake (8 messages) with a single
`RegisterCounterparty` per side, binding a local client to its counterparty
client id and **commitment (merkle) prefix** — the ICS-24 prefix under which that
counterparty stores its provable paths. On Stellar this is an `ibc-router` call;
on Cosmos it is `MsgRegisterCounterparty` (`ibc.core.client.v2`). The prefix is
`ibc` on the Stellar side and **empty** on the Cosmos side (Stellar SMT keys are
unprefixed). After both sides register, packets flow immediately — no version
negotiation, no port binding, no handshake.

### Flow 2 — Transaction model (prepare → sign → submit)

The transport mechanism beneath every ICS-26 message dispatch on the Stellar
side. Transactions are built where the chain connection lives (`interstellar-api`) and
signed where the key lives (the relayer), but **driven** by the relayer, so the
gateway never holds a key. The relayer sends an IBC message to the gateway; the
gateway asks the api to prepare an unsigned `tx_xdr`; the relayer signs it and
hands it back; the gateway submits it through the api to Soroban. Preparation is
method-agnostic, so every ICS message — `create_client` (ICS-02),
`register_counterparty` (ICS-26), `recv_packet` / `acknowledge_packet` /
`timeout_packet` (ICS-04), `update_client` (ICS-02) — flows through one path.

### Flow 3 — Stellar → Cosmos transfer · ICS-20 send + ICS-04 recv

1. **ICS-20 send.** `ibc-transfer.initiate_transfer(...)` runs `OnSendPacket`:
   escrows the asset via its **Stellar Asset Contract (SAC)** token contract
   (`transfer` under `require_auth`) and builds the `FungibleTokenPacketData`
   (`version = "ics20-1"`, `encoding = "application/json"`).
2. **ICS-04 send.** `ibc-router.send_packet(source_client, timeout, payloads[])`
   assigns the sequence and writes the **Packet Commitment** to the ICS-24
   commitment path in the SMT (`sha256` over the canonical packet fields, with the
   timeout hashed **big-endian** to match ibc-go).
3. The relayer observes the `send_packet` event (the gateway decodes the Soroban
   event; the `stellar-packet` worker decodes the v2 packet) and queries the
   **ICS-23 membership proof** of the commitment from the gateway.
4. **ICS-02 client update.** ibc-go rejects a proof newer than the client, so the
   worker first builds `MsgUpdateClient` advancing the destination `08-wasm`
   client to the proof height, then `MsgRecvPacket` (`ibc.core.channel.v2`), and
   submits `[update, recv]` together to Cosmos.
5. **On-chain verification.** ibc-go runs the `08-wasm` Stellar LC:
   `VerifyClientMessage` (SCP `EXTERNALIZE` statements → quorum → ledger) →
   `UpdateState`,
   then `VerifyMembership` (ICS-23 commitment proof vs `ConsensusState.root`). On
   success, ICS-04 writes the **receipt** + **acknowledgement commitment**, and
   the ICS-26 router invokes ICS-20 `OnRecvPacket`, which **mints the voucher** to
   the receiver and returns the success acknowledgement.

### Flow 4 — Ack-back leg · ICS-04 acknowledge + ICS-20 settle

The same worker chains straight into the acknowledgement direction so the source
commitment is cleared and the ICS-20 application settles:

1. **Extract the ack.** Read the application acknowledgement from the Cosmos
   recv-tx `write_acknowledgement` event — ibc-go v2 carries it as a proto
   `Acknowledgement { app_acknowledgements }` under `encoded_acknowledgement_hex`;
   for a successful ICS-20 recv the app ack is `{"result":"AQ=="}`.
2. Query the **ICS-23 membership proof** of the acknowledgement commitment from
   Cosmos (against its consensus root).
3. **ICS-02 client update.** Build `MsgUpdateClient` advancing the source
   `07-tendermint` client on Stellar to the ack proof height.
4. **ICS-04 acknowledge.** Build `MsgAcknowledgement` (`ibc.core.channel.v2`),
   routed through the Stellar endpoint → gateway `AckPacket` RPC →
   `ibc-router.acknowledge_packet`. The router recomputes the acknowledgement
   commitment over the app acks (`sha256` per ack, matching ibc-go), runs
   `VerifyMembership` via the `tendermint` LC, and **clears the packet
   commitment**.
5. **ICS-20 settle.** The ICS-26 router invokes ICS-20 `OnAcknowledgementPacket`:
   a success ack finalizes the escrow; a failure ack refunds it.

### Flow 5 — Timeout / refund · ICS-04 timeout + ICS-23 non-membership

If the destination never writes the receipt before the timeout, the relayer
proves **ICS-23 non-membership** of the receipt path (an absence proof against
the counterparty SMT root) and submits `timeout_packet` to the source
`ibc-router`. The router verifies the absence proof, clears the commitment, and
the ICS-26 router invokes ICS-20 `OnTimeoutPacket`, refunding the escrow to the
original sender.

{: .note }
> The reverse direction (Cosmos → Stellar) is symmetric: an ICS-20 `MsgTransfer`
> on Cosmos, ICS-04 `recv_packet` on the Stellar router (ICS-23 proof verified by
> the `tendermint` LC), ICS-20 mint/credit on Stellar, and the acknowledgement
> relayed back.

#### The gateway state tracker (why the ICS-23 proofs exist)

Every ICS-23 proof in Flows 3–5 depends on the gateway reconstructing the SMT at
a given height. The state tracker replays ledger close-meta **cumulatively**
(every ledger from the last processed up to the queried height, so the send
ledger's commitment write is ingested) and parses Soroban `TransactionMeta`
**V4** (the format the testnet emits). The proof is generated against the same
SMT root the `ConsensusState` carries, so the on-chain ICS-23 verify is
consistent.

---

## 4. How Stellar consensus is verified

This is the part with no established reference implementation to follow, so it is
worth setting out in full.

### Why Stellar is the harder direction

Tendermint has one globally agreed validator set with voting power attached, so
*"validators holding more than two-thirds of the power signed this block"* is a
well-defined statement, and the Soroban client checks exactly that.

Stellar has no equivalent. It uses **Federated Byzantine Agreement**: there is no
single configured validator set and no stake weighting. Each node chooses which
sets of peers it will listen to, and agreement is defined relative to those
choices. Four terms carry most of the weight:

| Term | Meaning |
|---|---|
| **slot** | one consensus instance — on Stellar, the slot number is the ledger sequence number |
| **quorum slice** | a set of nodes that is enough to convince *one particular node* of something |
| **quorum** | a set of nodes containing a slice for **every one of its own members** |
| **externalize** | the moment a node commits to a value for a slot, irreversibly — the event a light client must prove happened |

Two consequences shape the design. First, **there is no threshold to count to**:
the client has to evaluate whether a specific set of signers forms a quorum,
using the recursive structure Stellar validators publish. Observed on mainnet in
August 2026, that structure was *5 of 7 organizations, each 2 of 3 validators* —
21 signatures for a single ledger. Second, **the configuration is not carried in
the protocol**: a Tendermint client learns the next validator set from the chain
itself, and SCP offers nothing equivalent. The trusted configuration therefore
comes from outside, is governed deliberately, and — because it changes over
time — is stored with the range of ledgers each version applies to.

### Two claims, kept apart

```
SCP safety      → a quorum externalized value x for slot N   (what the protocol guarantees)
Ledger binding  → x is exactly the ledger header's own value (how stellar-core builds a ledger)
State binding   → that ledger commits to the IBC state root  (this project's construction)
```

Only the first is a property of SCP. A Cosmos chain gets the second and third
almost free, because its verified header carries the state root directly. On
Stellar they have to be built — and a quorum of valid signatures says nothing on
its own about which ledger or which state root travelled alongside it. Conflating
the two is the easiest way to end up with a client that performs real
cryptography and still proves nothing about the ledger.

### The checks, in order

| # | Check | In plain terms |
|---|---|---|
| L0 | Configuration applies | pick the trusted quorum configuration that governs this ledger; refuse if none does |
| L1 | Signatures | every message really was signed by the validator it names |
| L2 | Right kind of message | each is an `EXTERNALIZE` statement, for the slot being claimed |
| L3 | Quorum sets are genuine | each signer's published configuration matches the hash it committed to |
| L4 | The signers form a quorum | evaluated with Stellar's own recursive rule, not a count |
| L5 | They agree | all signers externalized the same value |
| L6 | The value is this ledger | the agreed value is byte-identical to the header's own field, so the header — and its hash — is authenticated |
| L7 | The ledger commits to the root | follow the header's commitment to transaction results down to the contract return value carrying the state root |

L0–L5 establish *that consensus happened*. L6–L7 establish *which ledger, and
which root*. Both are needed; neither implies the other.

### What this was checked against

The rules come from the SCP whitepaper (Mazières, *The Stellar Consensus
Protocol*, 2016). Wire-level details — the exact signing preimage, enum values,
structure layouts, and how a ledger header is built from an externalized value —
come from the `stellar-core` source and `stellar-xdr`, pinned to versions; those
are implementation facts, not protocol facts.

Neither was taken on trust. A reference checker fetches the archived SCP
messages, quorum sets and ledger header for a given ledger from Stellar's public
history archives and runs every check the contract must run:

- **L1–L6** were confirmed on live data from both networks — 21 signers under the
  nested mainnet configuration, 3 under a flat testnet one.
- **L7** was checked in two halves: the transaction-result hash matched the
  ledger header for **64 of 64** mainnet ledgers in a checkpoint, and the
  contract-return commitment matched the recorded hash for **40 of 40** Soroban
  invocations across eight consecutive testnet ledgers.

### What safety rests on

One assumption cannot be discharged on-chain: that the configured quorum
configuration is sufficiently related to the real Stellar network **for the
ledger in question**. That is checked off-chain with SDF's quorum-analysis
tooling before a client is created and at every configuration change, then
monitored for drift afterwards. Everything else — signatures, quorum evaluation,
parsing, and the two bindings — is implementation correctness, addressed by the
checks above and by testing against real ledgers.

A note on finality: a Stellar ledger is final once SCP externalizes the
corresponding value. The bridge requires no additional confirmations.

---

## 5. Deployment and infrastructure

Everything is driven by the `interstellar` CLI. A full local bring-up:

```sh
interstellar cosmos start --fresh     # local Cosmos devnet (ibc-go v11 simd + 08-wasm)
interstellar start --force-redeploy   # deploy contracts, upload the wasm LC, import relayer keys
interstellar clients cosmos           # 07-tendermint client on Stellar
interstellar clients stellar          # 08-wasm Stellar client on Cosmos
interstellar clients counterparty stellar
interstellar clients counterparty cosmos
interstellar transfer                 # originate a Stellar → Cosmos ICS-20 transfer
```

The moving parts run as containers — the Cosmos chain, the gateway, the api, and
the Hermes relayer — composed with healthchecks and dependency ordering so each
service waits for its dependencies to be ready.

Because the whole stack is Rust (contracts, core, gateway, api, wasm light
client, and the Hermes fork), the relayer integration is debuggable end-to-end in
one toolchain. The relayer inherits Hermes's event loop, transaction queueing,
client refresh, fee estimation, key management, and configuration unchanged; only
the chain-specific `StellarChainEndpoint` and the v2 `stellar-packet` worker are
added. The relayer's compatibility gates are widened to admit the ibc-go v11
`simd`, and `AnyClientState` / `AnyConsensusState` route the wasm light-client
envelope to the Stellar parser so Hermes tracks the `08-wasm` client like a
native one.

---

## 6. Architecture diagrams

### Component topology

```mermaid
flowchart TB
    subgraph Relayer["Hermes relayer (fork)"]
        EP["StellarChainEndpoint"]
        IC["ics10-stellar client types"]
        WK["stellar-packet worker"]
    end

    subgraph StellarSvc["Stellar off-chain services"]
        GW["interstellar-gateway (gRPC, no key) — StateTracker: SMT root + ICS-23 proofs"]
        API["interstellar-api (owns Soroban RPC + signing key) — tx prepare / submit / clients / events"]
    end

    subgraph StellarChain["Stellar / Soroban"]
        SOR["Soroban node (testnet or local)"]
        subgraph Contracts["Soroban contracts"]
            RT["ibc-router"]
            TR["ibc-transfer"]
            LC["light clients: tendermint · attestation · mock"]
        end
    end

    subgraph Cosmos["Cosmos counterparty (ibc-go v10+)"]
        WASM["08-wasm hosts light-client-wasm (Stellar LC)"]
    end

    Relayer -->|gRPC| GW
    Relayer -->|Tendermint RPC + gRPC| Cosmos
    GW -->|HTTP ApiClient| API
    API -->|Soroban JSON-RPC| SOR
    SOR --> Contracts
```

### Flow 1 — Counterparty registration · ICS-26 + ICS-24

```mermaid
sequenceDiagram
    autonumber
    actor Op as interstellar CLI
    participant RT as ibc-router (Stellar)
    participant CO as Cosmos (ibc-go)

    Note over Op,CO: ICS-26 RegisterCounterparty · binds client id + ICS-24 commitment prefix
    Op->>RT: register_counterparty(client_id, counterparty_client_id, prefix="ibc")
    RT-->>Op: registered
    Op->>CO: MsgRegisterCounterparty(client_id, counterparty_client_id, prefix=empty)
    CO-->>Op: registered
    Note over RT,CO: Both sides registered ⇒ packets flow (no handshake, no channels)
```

### Flow 2 — Transaction model (prepare → sign → submit)

```mermaid
sequenceDiagram
    autonumber
    participant R as Relayer
    participant GW as interstellar-gateway
    participant API as interstellar-api
    participant SOR as Soroban

    R->>GW: IBC message
    GW->>API: tx prepare (method args)
    API-->>GW: unsigned tx_xdr
    GW-->>R: unsigned tx_xdr
    Note over R: relayer signs tx_xdr with its key
    R->>GW: SubmitSignedTx(signed tx_xdr)
    GW->>API: tx submit
    API->>SOR: submit transaction
    SOR-->>API: result
```

### Flow 3 — Stellar → Cosmos transfer · ICS-20 send + ICS-04 recv

```mermaid
sequenceDiagram
    autonumber
    participant TR as ibc-transfer (Stellar)
    participant RT as ibc-router (Stellar)
    participant GW as gateway
    participant WK as stellar-packet worker
    participant CO as Cosmos (ibc-go)
    participant WASM as 08-wasm Stellar LC

    TR->>TR: ICS-20 OnSendPacket — escrow + build FungibleTokenPacketData
    TR->>RT: ICS-04 send_packet(source_client, timeout, payloads)
    RT->>RT: assign sequence, write ICS-24 Commitment → SMT
    GW-->>WK: observe send_packet (decoded event)
    WK->>GW: query ICS-23 membership proof (vs SMT root)
    GW-->>WK: proof + proof height
    WK->>CO: ICS-02 MsgUpdateClient(dest 08-wasm → proof height)
    WK->>CO: ICS-04 MsgRecvPacket (ibc.core.channel.v2)
    CO->>WASM: ICS-02 VerifyClientMessage (SCP quorum → ledger) → UpdateState
    CO->>WASM: ICS-23 VerifyMembership (commitment vs ConsensusState.root)
    WASM-->>CO: valid
    CO->>CO: write receipt + ack commitment
    CO->>CO: ICS-20 OnRecvPacket — mint voucher, return success ack
```

### Flow 4 — Ack-back leg · ICS-04 acknowledge + ICS-20 settle

```mermaid
sequenceDiagram
    autonumber
    participant CO as Cosmos (ibc-go)
    participant WK as stellar-packet worker
    participant EP as Stellar endpoint
    participant GW as gateway
    participant RT as ibc-router (Stellar)
    participant LC as tendermint LC (Stellar)
    participant TR as ibc-transfer (Stellar)

    Note over CO: continues from Flow 3 — recv succeeded
    WK->>CO: extract app ack from write_acknowledgement (encoded_acknowledgement_hex)
    Note over WK: ICS-20 success ack = {"result":"AQ=="}
    WK->>CO: query ICS-23 membership proof of the ack commitment
    CO-->>WK: ack proof + proof height
    WK->>EP: ICS-02 MsgUpdateClient(source 07-tendermint → ack proof height)
    WK->>EP: ICS-04 MsgAcknowledgement (ibc.core.channel.v2)
    EP->>GW: AckPacket RPC
    GW->>RT: ICS-04 acknowledge_packet
    RT->>RT: recompute ack commitment over app acks
    RT->>LC: ICS-23 VerifyMembership (vs ConsensusState.root)
    LC-->>RT: valid
    RT->>RT: clear packet commitment
    RT->>TR: ICS-20 OnAcknowledgementPacket — success finalizes escrow
    Note over WK,TR: round trip closed
```

### Flow 5 — Timeout / refund · ICS-04 timeout + ICS-23 non-membership

```mermaid
sequenceDiagram
    autonumber
    participant WK as stellar-packet worker
    participant CO as Cosmos (ibc-go)
    participant RT as ibc-router (Stellar)
    participant LC as tendermint LC (Stellar)
    participant TR as ibc-transfer (Stellar)

    Note over CO: destination did not write the receipt before timeout
    WK->>CO: query ICS-23 non-membership of the receipt path
    CO-->>WK: absence proof + proof height
    WK->>RT: ICS-04 MsgTimeout → timeout_packet
    RT->>LC: ICS-23 VerifyNonMembership (receipt absent vs ConsensusState.root)
    LC-->>RT: valid
    RT->>RT: clear packet commitment
    RT->>TR: ICS-20 OnTimeoutPacket — refund escrow to original sender
```
