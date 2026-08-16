---
title: Roadmap
layout: default
nav_order: 5
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

**`interstellar` orchestration CLI + a reusable multi-chain stack.** A single Rust
binary that deploys the contracts, uploads the Stellar `08-wasm` light client,
creates clients, registers counterparties, runs the relayer, and verifies Stellar
consensus against live archive data, no shell scripts.

- *How Stellar is used:* it drives the Soroban CLI, `interstellar-api`, and Docker
  to stand up and operate a complete Interstellar deployment reproducibly.
- *Impact:* the same protocol layer, relayer, and tooling extend to any future
  IBC counterparty, the marginal cost of the next chain is one light client + one
  endpoint, so the link scales O(n), not O(n²).

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
  consensus chain of custody.
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
  link by link against real archived ledgers, with a negative suite showing each
  check fails closed. The off-chain services assemble that evidence for the live
  relay path, and the relayer pins its trust root against a shipped constant
  rather than accepting quorum sets over the wire.
- **IBC v2 relayer:** a Stellar chain endpoint, Stellar client types, and a custom
  v2/Eureka packet-relay worker drive ICS-04 packet semantics, with migration to
  the proof-API-based IBC v2 relayer underway.

### Market validation

- **The interop market is enormous and the trust-minimized slice is unserved on
  Stellar.** IBC connects **115+ chains**, has moved hundreds of billions in
  cumulative volume, and has had **zero** protocol-level exploits since 2021. No
  trust-minimized, light-client-secured path in or out of Stellar exists today,
  existing Stellar bridges are federated/multisig.
- **The problem is expensive and proven.** The five largest bridge hacks (Ronin,
  Poly, Wormhole, Nomad, Harmony) all stem from the trusted-signer model IBC
  eliminates, and bridges remain the most-exploited category in crypto.
- **Stellar's distinctive value on the other side:** Stellar would be the only
  trust-minimized payments chain plugged into the largest interop graph, its
  anchors, regulated stablecoins, non-USD tokenized fiat, and cash-out network
  become reachable from 115+ chains, and vice versa.

---

## Stage 1: MVP (devnet)

**Goal:** Close the ICS-20 loop in both directions with the full consensus chain
of custody verified on-chain, on the devnet where the packet flows are already
proven.

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

**Deliverable 2. Full ICS-04 + ICS-20 round-trip, both directions.** Complete
ICS-04 packet semantics (`send` / `recv` / `acknowledge` / `timeout`) end-to-end
via the v2 packet-relay worker in both directions, closing the ICS-20 transfer
loop: outbound (escrow → received → acknowledged) and inbound (transfer →
credited on Stellar → acknowledged).
- *Completion criteria:* A single command runs a full round-trip in each
  direction on the devnet; relayer logs show the ack relayed back and the source
  commitment cleared; the acknowledgement check fails the run when no ack
  arrives.

**Deliverable 3. Real-asset transfers and the verification cost budget.**
ICS-20 escrow already runs over the canonical **Stellar Asset Contract (SAC)**
token interface, so this deliverable exercises it with real assets end to end,
native XLM and an issued stablecoin, and establishes the cost budget the design
depends on: measure the on-chain cost of verifying the consensus bundle and the
state-root proof on the host chain, and apply the batching mitigations if it
exceeds the budget (bind once per client update rather than per packet, and
chain headers backwards so one expensive step covers a range).
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
- *Dependency (third party):* requires a public IBC v2 chain with the `08-wasm`
  module to **approve and store the Stellar light client via on-chain
  governance**, a `store-code` proposal voted in by that chain's validators, with
  the checksum allow-listed in the `08-wasm` module. We de-risk by targeting
  operators amenable to new `08-wasm` clients and coordinating the proposal ahead
  of time (or using a permissioned testnet where we can drive the governance
  directly).

**Deliverable 2. A second counterparty, proving the artifact is portable.**
Deploy the *same* Stellar light-client wasm to a second, independent host chain
and complete ICS-20 transfers over it, with no changes to the Stellar-side
contracts and no new bridge code, the payoff of the reusable architecture.
- *Completion criteria:* the same client checksum stored on a second chain; both
  counterparties registered; ≥1 transfer per direction completed on testnet with
  on-chain proof verification; a written account of exactly what had to change
  (target: configuration only).
- *Dependency (third party):* the Stellar light client approved and stored on
  that chain via its governance process, coordinated ahead of time.

**Deliverable 3. Integration test suite + community testable build.** End-to-end
integration tests covering both corridors and edge cases (timeouts, failed acks,
client refresh, quorum-configuration rotation); a documented, reproducible
`interstellar` devnet/testnet build shared with the Stellar Discord for feedback.
- *Completion criteria:* Passing CI test suite; testable build + instructions
  shared in Discord; collected feedback summary.

---

## Stage 3: Mainnet

**Goal:** Ship to mainnet with production relayer operations, and meet the UX
readiness bar with both an end-user dApp and operator/integrator UX.

**Deliverable 1. Security review + hardening.** Internal security review of the
Soroban contracts, the wasm light client, and the relayer integration; fuzzing of
the SMT/proof paths; light-client lifecycle hardening; key-management and
operational-security review; remediation of findings from a third-party audit.
Includes standing up the quorum-configuration governance the trust model
requires: a named owner, validation with SDF's quorum-analysis tooling at client
creation and at every change, and drift monitoring.
- *Completion criteria:* Published internal review summary + fuzzing report;
  audit findings triaged and remediated; no open critical/high issues; the
  configuration process documented with an accountable owner.

**Deliverable 2. Mainnet launch + production relayer operations.** Deploy
contracts and light clients to Stellar mainnet; register mainnet counterparties;
run a production relayer with monitoring/alerting and rate limits.
- *Completion criteria:* Mainnet contract addresses; ≥1 live mainnet transfer per
  direction; relayer running stably ≥7 consecutive days with a monitoring
  dashboard.
- *Dependency (third party):* the **Stellar light client must be approved and
  stored on the target mainnet** via that chain's **on-chain governance**.
  Mainnet is additionally gated on the security review + external audit
  (Deliverable 1) completing with no open critical/high findings before any value
  moves.

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
