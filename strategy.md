---
title: Strategy
layout: default
nav_order: 3
description: >-
  Why the Interstellar project exists, what it builds, and the reasoning
  behind each architectural choice.
---

# Strategy
{: .no_toc }

Why this project exists, what we're building, and the reasoning behind each
architectural choice. Written for stakeholders, contributors, and anyone
evaluating Interstellar.

## Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## TL;DR

We are implementing **IBC v2 (Eureka)** for Stellar: not a bridge to a particular
chain, but a native implementation of the Interchain Standards on Soroban, plus
the Stellar light client that other chains load in order to verify Stellar for
themselves.

The result is that **Stellar gains first-class connectivity to every IBC-enabled
chain**, a network of **115+ chains** today, without relying on multisig
committees or federated validators. The marginal cost of the next counterparty is
a configuration entry, not an integration project.

IBC is increasingly **not a single-ecosystem protocol**. It is becoming the
generic interop substrate that lets independent chain families talk to *each
other* through shared, reusable parts. The same investment that connects Stellar
to its first counterparty ships the Stellar half of every future pairing, for
free, because the protocol layer is shared. That compounding (bridges that scale
O(*n*) instead of O(*n²*)) is what makes this work fundable as infrastructure,
not as a one-off.

---

## 1. Why connect Stellar to the IBC network

Stellar is exceptional at the **on-ramp / off-ramp / payment** layer:

- Sub-5-second settlement, sub-cent fees.
- A 10-year-old payments network with a global stablecoin and anchor
  ecosystem (USDC, EURC, MoneyGram, Circle, regional fiat anchors).
- The Stellar Development Foundation, AID-tech rails, and an embedded
  user base focused on remittances, FX, and tokenized real-world assets.
- Soroban smart contracts (Rust/WASM), the platform now supports the
  general-purpose programmability needed for IBC.

What it lacks is a way to move that value off-network without handing custody to
an intermediary. Every route out of Stellar today runs through a bridge operator,
a multisig, or an attestation service, and whoever that is becomes the weakest
point in the system.

**What IBC connectivity unlocks:**

- **Stellar-native assets reach the whole graph**, trust-minimized. USDC, EURC,
  XLM, and anchored RWAs become tradeable, lendable, and usable as collateral on
  115+ chains without wrapping through multiple bridges first.
- **Graph-native assets reach Stellar's payment rails**, spendable through the
  anchor network, cash-out points, and global remittance corridors.
- **Cross-chain settlement**, a remittance app can quote a rate on one chain,
  settle on Stellar, and pay out at a physical cash agent.
- **Soroban contracts callable from other chains**, composability across
  consensus layers rather than only asset movement.

The reach is not the whole argument, though. The argument is that this reach
arrives **without a new trusted party**, which is the thing no bridge Stellar can
use today offers.

---

## 2. Why IBC

There are at least a dozen cross-chain protocols (LayerZero, Wormhole,
Axelar, Hyperlane, Chainlink CCIP, deBridge, Synapse, Connext, …). We chose
**IBC** because it is the only one that is simultaneously:

**Trust-minimized.** IBC packet verification is performed by an **on-chain
light client** of the source chain running inside the destination chain.
There is no validator committee, no signing federation, no off-chain
multisig. The security of an IBC packet equals the security of the underlying
chains, nothing weaker.

Most "bridges" today rely on a permissioned set of signers who attest to
events. When that committee is compromised, funds are lost. The five largest
bridge hacks in crypto history (Ronin, Poly, Wormhole, Nomad, Harmony) all
share this pattern. IBC's light-client model makes that class of failure
impossible.

**Battle-tested.** IBC has moved hundreds of billions of dollars in cumulative
volume since 2021 with no consensus-level exploit, across 115+ connected chains.
It is the most-used cross-chain protocol by transaction count.

**Standard.** IBC is a public, open spec (the
[ibc-protocol.org](https://ibc-protocol.org/) standards). Implementations
exist in Go, Rust, Solidity, and Move. Adopting IBC means Stellar can talk
to *any* IBC-enabled chain (present or future) without bespoke per-pair
integration work. Network effect compounds.

**Aligned with Stellar's values.** Stellar was founded on a "trust anchor"
model: trust is explicit, verifiable, and revocable. IBC's light-client
model is the cross-chain expression of that same principle. It is the
ideologically correct choice for a chain whose founding ethos rejects
opaque intermediaries.

**Composable upgrade path.** Through **ICS-20** (transfer) we get fungible
token transfers; through **ICS-27** (interchain accounts) we get
cross-chain smart-contract calls; through **08-wasm** we get pluggable
light clients without forking the host chain. The same primitive
extends from "move USDC" to "trigger a Soroban contract from another chain."

---

## IBC as a generic interop state machine

A broader framing worth making explicit, because it changes what kind
of investment this work represents.

IBC is, at its core, a **blockchain-interop state machine**. The
specification dictates what state must be provable (commitments,
receipts, ack commitments), what must be verifiable (header /
membership / non-membership proofs), and what the packet lifecycle
looks like. It deliberately says nothing about *how* those state
transitions are computed underneath. **IBC can be implemented under
any virtual machine, any consensus algorithm, any programming
language**, wherever you can hash, sign, and verify a Merkle proof.

The most mature implementation today grew up in one ecosystem, but that is an
accident of timing, not a property of the protocol. Increasingly, IBC is being
adopted as the **default interoperability substrate** by chain families with
entirely different consensus and execution models, Polkadot/Substrate
implementations in production, Ethereum rollups onboarding through Eureka, and
active integration work for Solana and Near among others. The more independent
chain families speak it, the more value it provides as a shared language between
them.

That is precisely why this project is possible. Stellar's consensus looks nothing
like the chains IBC grew up on, and Soroban's execution model looks nothing like
theirs either. Neither matters to the protocol. What matters is that Stellar can
commit to the three provable paths, and that another chain can verify Stellar's
consensus. Both are engineering problems with answers.

Once two chains both speak IBC, they talk **directly to each other** through the
same packet protocol. No third chain has to sit in the middle, no new bridge has
to be built. The day a Stellar light client exists on a chain that already has a
counterparty client, that pair is bridged. **Marginal cost of the next chain pair
approaches zero.**

### Bespoke bridges scale O(n²); IBC scales O(n)

The dominant model in crypto interop today is per-pair custom bridges.
It is also the dominant *cost*. For *n* chains to be pairwise bridged
with custom code, the industry has to build, audit, and operate ~n²/2
bridges:

```
3 chains   →    3 bridges
10 chains  →   45 bridges
20 chains  →  190 bridges
50 chains  → 1225 bridges
```

Each one needs its own security model, audited codebase, operator set,
relay incentive design, and ongoing maintenance. Industry-rough
numbers put the build-and-audit cost in the hundreds of thousands to
low millions per bridge, with significant ongoing operating cost, and
custom bridges remain the single most-exploited category of crypto
infrastructure by total value lost.

IBC inverts the topology. With *n* chains all speaking IBC, you need:
- *n* light clients (one per chain, packaged as `08-wasm` blobs and
  embedded wherever they need to be verified, write once, run
  anywhere).
- **1** shared protocol specification.
- **1** generalized relayer stack, with an abstract chain-endpoint
  interface that every chain implements.

The marginal cost of the (*n*+1)-th chain is **one light client + one
chain endpoint**. Not *n* new bridges. The investment in the shared
core, the Stellar relayer endpoint, and `light-client-wasm` is reused by
every future chain pair this stack serves.

### The client is a portable artifact

The detail that makes those economics real rather than rhetorical: the Stellar
light client is a **wasm blob, not a deployment**.

Through the `08-wasm` client module, a host chain loads a light client as data.
No fork of the host binary, no coordinated upgrade of its codebase. The same
artifact that lets one chain verify Stellar lets **every other `08-wasm`-capable
chain** verify Stellar, with no additional Stellar-side work.

So "one light client per chain" is not one light client per *pair*. It is one per
chain, written once, hosted anywhere. That is the mechanism by which the marginal
cost of the next counterparty approaches configuration rather than engineering.

### Why this matters for funding

This framing changes the pitch substantially. A grant for "another Stellar
bridge" reads as a **point investment** with a single use case. A grant for
**"the Stellar implementation of a generic, reusable, multi-chain interop state
machine, designed to support every chain family that joins"** reads as
**infrastructure investment** with compounding returns.

The same dollar funds:

- A working trust-minimized IBC link for Stellar (the immediate, demonstrable
  deliverable).
- The Stellar side of every future pairing that emerges in the next several
  years, for free, because the protocol layer is shared.
- A maintained Stellar chain endpoint in the shared relayer, permanent
  infrastructure for every IBC operator running Stellar.
- Validation that the chain-endpoint pattern generalizes across consensus
  families, which strengthens the case for further integrations and lowers the
  bar for any of them.

Bespoke bridge funding terminates at the boundary of its chain pair.
IBC funding **compounds across the interop graph**. That is the
strongest single argument we have to make.

---

## 3. Why build on proven IBC infrastructure

Extending IBC to a chain family it did not grow up in is not speculative work.
The pattern (an abstract relayer chain endpoint, plus a wasm light client for a
consensus family outside the one IBC was designed around, plus devnet
orchestration tooling) has been built and shipped before, and this project builds
on those foundations directly rather than reinventing them.

That prior art matters for three concrete reasons:

1. **Time to market.** Building the relayer abstraction, light-client packaging,
   and orchestration tooling from scratch would take 18–24 months. Reusing proven
   foundations compresses it to months.
2. **Evidence the architecture generalizes.** A second independent consensus
   model implemented against the same interfaces is the strongest available
   signal that the abstraction is real, not incidental to one chain. Patterns
   that work for one consensus family inform the next.
3. **Shared maintenance.** Improvements to the shared relayer infrastructure
   benefit every chain using it, and the cost of tracking upstream is amortized
   across more than one team.

The commitment-tree shape is shared for the same reason: a fixed-depth-64 Sparse
Merkle Tree with common leaf, inner, and index rules means proofs interoperate
without a bespoke verifier per pair.

---

## 4. The relayer: Hermes today, the IBC v2 relayer next

The link runs end to end today on a fork of **Hermes**. It is being migrated to
the **IBC v2 relayer**, for one architectural reason above all: that relayer does
not embed chain-specific proof logic, it obtains proofs from a separate proof API
over gRPC, which the project's gateway already implements. The proof half of the
integration is therefore already done, with no fork required. What the migration
adds is a Stellar chain type: transaction construction and submission, signing,
and a finality rule.

The general principle: **chain-specific logic belongs behind an interface, not
inside a relayer fork.** That is what makes the relayer layer genuinely shared
infrastructure rather than *n* forks in a trenchcoat.

The reasoning that led to Hermes in the first place still explains the shape of
the current implementation. Hermes is the **reference Rust IBC relayer**
maintained by Informal Systems. We chose it over the alternatives (the Go relayer
`rly`, ts-relayer, custom code) because:

**Maturity.** Hermes has been the workhorse of IBC relaying since 2021. It
runs in production relaying significant TVL daily. Bug surface is well
understood; failure modes are documented.

**Rust-native.** The rest of our stack is Rust (Soroban contracts,
shared core, gateway, api, light-client-wasm). Hermes lets us
contribute upstream and debug across the entire stack in a single
language and toolchain. No FFI, no cross-language impedance mismatch.

**Forkable, extensible architecture.** Hermes splits chain logic behind
a `ChainEndpoint` trait, so a new chain family is added by implementing that
trait rather than by rewriting the relayer. The pattern is proven and the
codebase invites it. Rolling our own relayer would mean re-implementing event
subscription, transaction queuing, light-client update logic, packet timeouts,
fee estimation, key management, and configuration, all of which Hermes solves.

**IBC v2 support.** Recent Hermes versions support the v2 packet
lifecycle natively. Less code to write, less risk of spec deviation.

**Operator ecosystem.** IBC relayer operators already know how to run Hermes.
When the link goes live, the operator-facing surface (`hermes start`,
`~/.hermes/config.toml`, `keys add`, `query packet …`) is familiar.

---

## 5. Why IBC v2 instead of v1

IBC v2 (Eureka) ships a dramatically simpler protocol than v1, and we
benefit on every axis.

**No handshakes.** V1 requires a multi-step `Connection*` + `Channel*`
handshake to establish a route between two chains, four messages per
side, with both sides needing to be live during the ceremony. V2
collapses this to **one call**: `registerCounterparty(clientId,
merklePrefix)`. Bringing a new chain pair online goes from a
multi-hour, multi-party operation to a single transaction per side.

**Three provable paths instead of eight.** V1 mandates a Merkle store
with 8 specific paths (`clients/`, `connections/`, `channels/`,
`commitments/`, `acknowledgements/`, `receipts/`, `nextSeqRecv/`, etc).
V2 keeps only the 3 that matter for packet relay:

```
Packet Commitment      = {sourceClientId} || 0x01 || be64(seq)
Packet Receipt         = {destClientId}   || 0x02 || be64(seq)
Acknowledgement Commit = {destClientId}   || 0x03 || be64(seq)
```

For Stellar this is decisive: Soroban storage is **expensive** (state
rent based on byte-count). Fewer paths means dramatically lower
cost-per-packet. It also keeps the light client small enough to run inside
another chain's gas budget: 3 path patterns to verify instead of 8, and a
fixed-shape commitment tree that would not have been viable under v1.

**Per-packet app routing.** V1 binds an application (port) to a channel
at handshake time. V2 puts `sourcePort` / `destPort` into the packet
payload itself. A single client pairing can carry packets for any
number of applications, including future apps we haven't deployed yet.
Token transfer is the first application, not the shape of the integration.

**Cheaper, faster, simpler client lifecycle.** No `ChannelOpenInit/Try/
Ack/Confirm`, no version negotiation, no port binding. The gateway's
query service doesn't even implement `QueryClientState`,
`QueryConsensusState`, or `QueryNextSeqRecv`, all three are
non-provable in v2.

**Greenfield advantage.** V2 is the right call for a chain with no v1
legacy. We pay no migration tax, no backward-compatibility cost. The
ecosystem direction is clearly v2, so adopting it means Stellar plugs into the
*current generation* of the IBC graph, not the legacy one.

**Smaller attack surface.** Less protocol surface = less to audit, less
to get wrong, less for adversaries to probe. For a system whose security claim
rests on implementation correctness, that is a security argument, not just an
engineering convenience.

---

## 6. Why this is beneficial

### For Stellar

**Immediate liquidity reach.** The day the link goes live, 115+ IBC-enabled
chains become destinations for Stellar's stablecoins (USDC, EURC) and native
assets (XLM), and every asset on the graph can flow into Stellar's payment
network. Each chain that joins IBC later is reachable at near-zero marginal cost.

**A security claim that can be defended technically.** "No committee holds your
funds" is a statement that survives scrutiny, and it differentiates Stellar
against every committee-based bridge in the space.

**Validation of Soroban as a serious smart-contract platform.**
Implementing a non-trivial cross-chain protocol on Soroban (deterministic SMT,
ICS-23 proof verification, light-client contracts) demonstrates that Soroban is
production-ready for systems work, not just simple token logic.

**Differentiator vs other L1 payment networks.** Most payment-focused
chains (Ripple, Algorand, Hedera) have either bespoke bridges or rely
on federated message buses. Stellar with IBC becomes the *only*
trust-minimized payment chain plugged into the largest interop graph
in crypto.

**Real-world asset (RWA) corridor.** Stellar anchors tokenize fiat, gold, and
real estate. IBC lets those tokenized RWAs reach DeFi venues (margining,
lending, derivatives) without leaving a trust-minimized custody path.

**No moat erosion.** Stellar's existing strengths (fast/cheap settlement, anchor
network, regulated stablecoins) are amplified by IBC, not replaced. Each chain
stays focused on what it does best and becomes a distribution channel for the
others.

**Soroban contracts callable from another chain.** ICS-27 interchain
accounts and ICS-31 cross-chain queries (both v2-compatible) let a
contract elsewhere trigger Soroban execution. Stellar becomes a
*destination for cross-chain logic*, not just an end-point.

### For the wider IBC ecosystem

**Cross-chain stack validation.** Every consensus family added to the shared
relayer and light-client patterns is evidence that the architecture generalizes.
Pattern reuse across independent chain families is the strongest signal that an
abstraction is real, and it lowers the bar for every subsequent integration.

**Shared maintenance burden.** More teams working on the same relayer
infrastructure yields more fixes, more features, better test coverage, and faster
upstream merges. Every participant's investment compounds.

**A payments-native member of the graph.** IBC gains direct access to regulated
stablecoins, tokenized RWAs, non-USD fiat tokens, and a cash-out network, asset
classes and rails the graph is comparatively thin on, supplied natively rather
than wrapped.

---

## In one paragraph

Cross-chain infrastructure today mostly asks users to trust an operator, and that
is where the industry's losses have come from. IBC removes the operator by having
each chain verify the other's consensus in on-chain code, and it is a general
interop state machine rather than a single-ecosystem transport, implementable
under any VM and any consensus algorithm. Implementing it for Stellar means
Stellar joins a protocol rather than gaining a bridge: one light client, written
once and loadable by any host chain, makes 115+ current chains and every future
one reachable with no new trusted party per pair. The hard part is verifying
Stellar's consensus from the outside, because Stellar has no global validator set
to count and no header field binding an agreed value to application state. That
problem is solved, implemented on-chain, and validated against live mainnet data.
We did not have to invent the protocol, the relayer, or the orchestration. We had
to implement the Stellar-shaped piece, and that is exactly what this project
does.
