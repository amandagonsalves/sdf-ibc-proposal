---
title: Roadmap
layout: default
nav_order: 7
description: >-
  What the Interstellar project delivers, what already works, and the
  staged path from devnet to mainnet.
---

# Roadmap
{: .no_toc }

What the project delivers, what already works today, and the staged path from
devnet through public testnet to mainnet. Each deliverable is tracked against the
Interchain Standards the stack implements.

## Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Products & services

**On-chain IBC v2 light-client verification on Soroban.** A full IBC v2 (Eureka)
protocol stack as Soroban contracts implementing the Interchain Standards: an
`ibc-router` (**ICS-26** routing + **ICS-04** packet semantics), an inbound light
client (**ICS-02**), a deterministic fixed-depth-64 Sparse Merkle Tree for the
**ICS-24** host paths whose root is maintained on-chain, and an **ICS-23**
membership/non-membership proof serializer.

- *How Stellar is used:* counterparty packet commitments, receipts, and
  acknowledgements are committed to the SMT and verified **on-chain by Soroban
  contracts** (`VerifyClientMessage`/`UpdateState`,
  `VerifyMembership`/`VerifyNonMembership`), no multisig committee, no federated
  signers; packet security equals the security of the connected chains.
- *Impact:* this is the verification root everything else depends on. It makes
  Stellar a first-class IBC chain and proves Soroban is production-ready for
  serious systems work (on-chain SMT + proof verification).

**The Stellar light client, loadable by any IBC host chain.** `light-client-wasm`
compiled to wasm and deployed through the **08-wasm** module, verifying SCP
`EXTERNALIZE` statements, evaluating the quorum, and binding the agreed value to
a ledger and that ledger to the IBC state root.

- *How Stellar is used:* it is the artifact that lets **any** of the 115+ IBC
  chains verify Stellar for itself, with no fork of the host chain and no
  Stellar-side work per counterparty.
- *Impact:* this is what converts the project from a bridge into protocol
  membership. Write the client once, and every current and future IBC host can
  run it.

**Trust-minimized cross-chain transfers (ICS-20) with a relayer.** An
`ibc-transfer` Soroban app plus a Stellar chain endpoint in the shared relayer,
fronted by a gRPC `gateway` and an HTTP `api` that build unsigned Soroban
transactions the relayer signs and submits. Neither service holds a signing
key.

- *How Stellar is used:* the transfer app runs the **ICS-20** routing callbacks
  (`OnSendPacket` escrow, `OnRecvPacket` mint/credit, `OnAcknowledgementPacket`
  settle, `OnTimeoutPacket` refund) over **ICS-04** packets, moving real Stellar
  assets (XLM, USDC, EURC) through the **Stellar Asset Contract (SAC)** token
  interface.
- *Impact:* Stellar stablecoins and native assets reach the entire IBC graph, and
  IBC-native assets reach Stellar's payment and anchor rails, both directions,
  trust-minimized.

**A Stellar chain type in the Cosmos IBC v2 relayer, plus the proof API behind
it.** The link runs today on a Hermes fork; the destination is the **Cosmos IBC
v2 relayer**, which takes proofs from a separate **proof API** over gRPC instead
of embedding chain-specific proof logic. `interstellar-gateway` already serves
that proof API, so the Stellar-specific work sits behind a service boundary this
project owns rather than inside a fork.

- *How Stellar is used:* the chain type constructs and submits Soroban
  `InvokeHostFunction` transactions, signs through a remote signer backed by
  `interstellar-api` or a local key, and applies Stellar's finality rule (a
  ledger is final once SCP externalizes its value, with no extra confirmations).
- *Impact:* Stellar becomes a chain type in the relayer every IBC operator
  already runs, rather than a fork one team maintains. It retires the cost of
  tracking upstream Hermes, and it is the piece that makes "run a Stellar
  relayer" an ordinary operational choice for third parties.

**`interstellar` orchestration CLI, and a standalone consensus verifier.** A
single Rust binary that deploys the contracts, uploads the Stellar `08-wasm`
light client, creates clients, registers counterparties, runs the relayer, and
verifies Stellar consensus against live archive data, no shell scripts. The same
verifier also ships as a **standalone crate** that can be read, run and audited
without the rest of the project.

- *How Stellar is used:* it drives the Soroban CLI, `interstellar-api`, and Docker
  to stand up and operate a complete Interstellar deployment reproducibly, and it
  runs the contract's own verification primitives against real history-archive
  data, naming the step that fails.
- *Impact:* the same protocol layer, relayer, and tooling extend to any future
  IBC counterparty, the marginal cost of the next chain is one light client + one
  endpoint, so the link scales O(n), not O(n²). The verifier makes the central
  security claim **falsifiable by a reviewer** rather than asserted in a diagram.

**End-user transfer dApp + operator/integrator tooling.** A web app that turns the
transfer flow into a one-click product with a live status stepper, plus an
operator runbook, integrator guide, and monitoring dashboard.

- *How Stellar is used:* the dApp signs the `initiate_transfer` Soroban
  invocation with a browser wallet and tracks the resulting voucher; nothing is
  custodial and no key leaves the user.
- *Impact:* lowers adoption cost, any Stellar app can plug into the transfer
  flow, and any operator can run an Interstellar relayer.

---

## What already works

A pre-launch infrastructure project's traction is **technical proof + market
validation**, not live user metrics yet.

### Technical traction

Already built and demonstrably working, tracked against the Interchain Standards
the stack implements. Interstellar runs on a local devnet against a real IBC v2 +
`08-wasm` host chain:

- **ICS-26 (Routing), done.** The `ibc-router` Soroban contract dispatches
  `send` / `recv` / `ack` / `timeout`, and IBC v2 counterparty registration
  (`registerCounterparty`) is complete on both sides, no v1 connection or channel
  handshake.
- **ICS-24 (Host requirements), done.** Packet commitment, receipt, and
  acknowledgement paths live in a deterministic fixed-depth-64 Sparse Merkle
  Tree, byte-exact against the reference implementation, whose root is recomputed
  and published **on-chain** on every provable write.
- **ICS-02 (Client semantics), both clients implemented on-chain.** An inbound
  light client on the Stellar router, and the Stellar light client compiled to
  wasm and uploaded to the host chain via `MsgStoreCode`, implementing the full
  consensus chain of custody. Lifecycle hardening on the inbound client is
  itemized in Stage 3.
- **ICS-23 (Vector commitments), done.** Membership (for `recv`/`ack`) and
  non-membership (for `timeout`) proof verification over the SMT.
- **ICS-04 + ICS-20, packet flows demonstrated end to end.** `interstellar
  transfer` escrows and emits a `SendPacket`; the relayer fetches the commitment
  proof and submits the receive; on-chain verification passes and the host chain
  credits the receiver with a success acknowledgement, which is relayed back and
  verified on Stellar, closing the round trip.
- **Stellar consensus verification, validated against live mainnet data.** The
  full chain (signatures, quorum evaluation, agreement, ledger binding, and the
  binding from ledger to IBC state root) is implemented on-chain and confirmed
  link by link against real archived ledgers: **64 of 64** mainnet ledgers in a
  checkpoint for the transaction-result binding, **40 of 40** Soroban invocations
  for the state-root binding, and an **18-case negative suite** showing each
  check fails closed. The off-chain services assemble that evidence for the live
  relay path, and the relayer pins its trust root against a shipped constant
  rather than accepting quorum sets over the wire.
- **Relaying, with the migration already half-landed.** A Stellar chain endpoint,
  Stellar client types, and a custom v2/Eureka packet-relay worker drive ICS-04
  packet semantics on the Hermes fork. On the Cosmos IBC v2 relayer side, the
  proof API is **already served by the gateway** and both chains plus the routing
  are configured; what remains is the Stellar chain type itself.

The measurable shape of that work: **529 commits** and **29,730 lines of Rust**
in the main repository, **7,916 lines** of Stellar code in the relayer fork, a
**3,304-line standalone consensus verifier** that can be audited on its own, and
**316 tests passing** across the two cargo workspaces. Deployed artifacts are
small because IBC v2 is small: 27 KB for the router, 18 KB each for the transfer
app, the voucher token and the inbound client, and 469 KB for the host-chain
Stellar light client. Full breakdown on
[Implementation & Evidence](implementation.html).

### Market validation

- **The interop market is enormous and the trust-minimized slice is unserved on
  Stellar.** IBC connects **115+ chains**, has moved hundreds of billions in
  cumulative volume, secures roughly **$40B a year**, and has had **zero**
  protocol-level exploits since 2021. No trust-minimized, light-client-secured
  path in or out of Stellar exists today, existing Stellar bridges are
  federated/multisig.
- **The problem is expensive and proven.** The five largest bridge hacks (Ronin
  $625M, Poly $611M, Wormhole $326M, Nomad, Harmony) all stem from the
  trusted-signer model IBC eliminates. Bridges account for **$2.5B+** in losses
  and were roughly **70% of all crypto theft in 2022**, and they remain the
  most-exploited category in crypto.
- **There is a specific first market, not a generic one.** Stellar is the only
  chain carrying tokenized **non-USD currency** at meaningful scale (naira,
  reais, pesos, shillings) while the IBC graph is almost entirely
  dollar-denominated. Those markets stay thin because a market maker holding a
  regional currency has nowhere to hedge the risk; the graph supplies the spot
  depth and derivatives venues that make hedging possible, and Stellar supplies
  the assets. Neither side can develop that market alone, which gives an
  identifiable class of counterparty to approach on day one. The full argument,
  and the fair objection to it, are on
  [Questions & Objections](questions.html#what-is-the-use-case).
- **Stellar's distinctive value on the other side:** Stellar would be the only
  trust-minimized payments chain plugged into the largest interop graph, its
  anchors, regulated stablecoins, non-USD tokenized fiat, and cash-out network
  become reachable from 115+ chains, and vice versa.
- **The buyer's objection is regulatory, and IBC is the answer to it.** A
  tokenized-treasury issuer or licensed anchor operates under legal opinions
  governing who may hold the asset. A committee bridge introduces a custodian
  those issuers have not vetted and frequently cannot accept; the absence of a
  trusted intermediary is not a preference for them but the only structure that
  clears compliance.

---

## Stage 1: MVP (devnet)

**Goal:** Close the ICS-20 loop in both directions with the full consensus chain
of custody verified on-chain, and complete the relayer migration, on the devnet
where the packet flows are already proven.

**Deliverable 1. Consensus verification proven in the live packet flow.** The
off-chain services already assemble the evidence bundle the on-chain Stellar
light client expects (SCP `EXTERNALIZE` envelopes, quorum sets, the next slot's
transaction set, and the state-root proof), and the relayer pins its trust root
against a shipped constant. This deliverable proves it end to end in the relay
path and closes the operational gaps around it: a caller for the root-refresh
entrypoint so an idle link can still bind a recent root, and removal of the
development-only light clients from the default deployment.
- *Completion criteria:* a devnet transfer whose client update carries a real
  consensus bundle and is accepted on-chain; a mutated bundle is rejected; a
  transfer succeeding after an idle period; no always-accept client registered
  after a default deploy; end-to-end run reproducible from a single command.

**Deliverable 2. Migration to the Cosmos IBC v2 relayer.** Retire the Hermes fork
by moving the link onto the **Cosmos IBC v2 relayer**, which obtains proofs from
a proof API over gRPC rather than embedding chain-specific proof logic. The proof
half is already done: `interstellar-gateway` serves that API on the same port as
its other services, and the relayer already runs under its own compose profile
with Postgres, both chains configured, and the routing authored (client
identifiers per direction, per-direction batch sizes, and an acknowledgement
policy that relays success acks rather than dropping them, because ICS-20 escrow
only settles when the success ack gets home).

What this deliverable adds is the **Stellar chain type** in the relayer itself.
Its chain-type abstraction covers Cosmos, EVM and SVM today, and an entry of any
other type is skipped when the bridge clients are built, so the Stellar routing
is currently correct but **inert**. The work is transaction construction and
submission as Soroban `InvokeHostFunction` invocations, signing through either a
remote signer backed by `interstellar-api` or a local key, and the finality rule
(a Stellar ledger is final once SCP externalizes its value). Stellar-bound
batches stay pinned at one packet while Soroban allows a single host-function
invocation per transaction and `recv_packet` takes one packet; batching applies
normally in the other direction.
- *Completion criteria:* a full devnet round trip relayed end to end by the
  Cosmos IBC v2 relayer with no Hermes fork in the path; proofs sourced from the
  gateway's proof API; the trust-root fingerprint check enforced at startup and
  failing loudly on mismatch; the fork archived and upstream-tracking work
  stopped.

**Deliverable 3. Full ICS-04 + ICS-20 round-trip, both directions.** Complete
ICS-04 packet semantics (`send` / `recv` / `acknowledge` / `timeout`) end-to-end
via the v2 packet-relay worker in both directions, closing the ICS-20 transfer
loop: outbound (escrow → received → acknowledged) and inbound (transfer →
credited on Stellar → acknowledged).
- *Completion criteria:* A single command runs a full round-trip in each
  direction on the devnet; relayer logs show the ack relayed back and the source
  commitment cleared; the acknowledgement check fails the run when no ack
  arrives.

**Deliverable 4. Real-asset transfers and the verification cost budget.**
ICS-20 escrow already runs over the canonical **Stellar Asset Contract (SAC)**
token interface, so this deliverable exercises it with real assets end to end,
native XLM and an issued stablecoin, and establishes the cost budget the design
depends on: measure the on-chain cost of verifying the consensus bundle and the
state-root proof on the host chain, and apply the batching mitigations if it
exceeds the budget (bind once per client update rather than per packet, and
chain headers backwards so one expensive step covers a range).

This is the **dominant unknown** in the whole design. The transaction result set
is the term that drives it, because `txSetResultHash` is a flat hash rather than
a Merkle root and the whole set therefore travels with the header (see
[Architecture § 8](architecture.html#8-performance)).
- *Completion criteria:* A transfer of a real SAC asset escrowed on send and
  released on a successful ack, plus a timeout-refund path, with tx hashes; a
  published per-update cost measurement against a host chain's limits, and a
  documented batching strategy if the measurement requires one.

---

## Stage 2: Testnet

**Goal:** Run continuously on public testnets, prove the architecture generalizes
by adding a second host chain, and open it to community testing.

**Deliverable 1. Public testnet deployment + continuous relayer ops.** Deploy
contracts to Stellar testnet, connect to a public IBC v2 testnet counterparty,
run the relayer continuously with monitoring/alerting, and publish operator
documentation. Transfers observable on public explorers.
- *Completion criteria:* Public contract addresses; a live relayer running ≥7
  days; ≥1 transfer per direction visible on stellar.expert + the counterparty's
  explorer; operator runbook published.

**Deliverable 2. A second counterparty, proving the artifact is portable.**
Deploy the *same* Stellar light-client wasm to a second, independent host chain
and complete ICS-20 transfers over it, with no changes to the Stellar-side
contracts and no new bridge code, the payoff of the reusable architecture.
- *Completion criteria:* the same client checksum stored on a second chain; both
  counterparties registered; ≥1 transfer per direction completed on testnet with
  on-chain proof verification; a written account of exactly what had to change
  (target: configuration only).

**Deliverable 3. Integration test suite + community testable build.** End-to-end
integration tests covering both corridors and edge cases (timeouts, failed acks,
client refresh, quorum-configuration rotation, and recovery by re-fetching a
checkpoint after an outage); a documented, reproducible `interstellar`
devnet/testnet build shared with the Stellar Discord for feedback.
- *Completion criteria:* Passing CI test suite; testable build + instructions
  shared in Discord; collected feedback summary.

---

## Stage 3: Mainnet

**Goal:** Ship to mainnet with production relayer operations, and meet the UX
readiness bar with both an end-user dApp and operator/integrator UX.

**Deliverable 1. Security review + hardening.** Internal security review of the
Soroban contracts, the wasm light client, and the relayer integration; fuzzing of
the SMT/proof paths; key-management and operational-security review; remediation
of findings from a third-party audit. Includes standing up the
quorum-configuration governance the trust model requires: a named owner,
validation with SDF's quorum-analysis tooling at client creation and at every
change, and drift monitoring. This is the one assumption no contract can
discharge, so it gets an owner rather than a footnote.

It also closes the **client-lifecycle hardening** on the inbound Soroban client,
named precisely so it can be checked off rather than left as a general aspiration
(see [Architecture § 6](architecture.html#hardening-still-in-progress)): read the
configured trust level instead of assuming two-thirds, enforce the stored
unbonding period and maximum clock drift, check the trusting period at proof
verification and not only at update, enforce the client state's ICS-23 proof
specs during verification, and gate the misbehaviour path on a header that has
passed the six checks first. None of these weaken the signature checking or the
threshold tally today; they are the difference between a client that verifies
correctly and one whose security parameters are exactly what a deployment
instantiated it with.

- *Completion criteria:* Published internal review summary + fuzzing report;
  audit findings triaged and remediated; no open critical/high issues; each
  hardening item above closed with a test; the configuration process documented
  with an accountable owner.

**Deliverable 2. Mainnet launch + production relayer operations.** Deploy
contracts and light clients to Stellar mainnet; register mainnet counterparties;
run a production relayer with monitoring/alerting and rate limits. Mainnet is
gated on Deliverable 1 completing with no open critical/high findings **before
any value moves**.
- *Completion criteria:* Mainnet contract addresses; ≥1 live mainnet transfer per
  direction; relayer running stably ≥7 consecutive days with a monitoring
  dashboard.

**Deliverable 3. End-user transfer dApp (UX readiness).** Connect a Stellar
wallet and a counterparty wallet, enter amount + receiver, sign
`initiate_transfer`, and watch a status stepper (`escrowed → relaying →
received`) as the voucher appears, plus a `GET /config` api endpoint so nothing
is hardcoded. Includes onboarding (wallet setup guidance, test-token button),
error handling, and an FAQ.
- *Completion criteria:* Live demo URL; screen recording of a full transfer;
  onboarding flow + FAQ present; works against testnet and (post-mainnet)
  mainnet.

**Deliverable 4. Operator/integrator UX + documentation.** Polished `interstellar`
operator UX, an operator runbook (run your own Interstellar relayer), an integrator
guide (plug an app into the transfer flow), a public monitoring dashboard, and a
published docs site.
- *Completion criteria:* Docs site live at a public URL; operator + integrator
  guides published; dashboard URL showing relay/transfer activity.
