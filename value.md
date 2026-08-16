---
title: Value & Comparison
layout: default
nav_order: 2
description: >-
  What Stellar gains from IBC connectivity, the network it reaches, and how
  IBC's trust-minimized model compares to the bridges Stellar could use
  instead.
---

# Value & Comparison
{: .no_toc }

What connecting Stellar to IBC is actually worth, the value it brings *into*
the Stellar ecosystem, and why a trust-minimized IBC path is materially better
than the bridges Stellar could otherwise use.

## Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## The thesis

Stellar already holds enormous transactable value, over **$800M in USDC**,
**$2B+ in on-chain real-world assets**, and billions in monthly stablecoin
payment volume. What it lacks is a way to move that value to other chains
*without handing custody to a trusted intermediary.*

Every bridge available to Stellar today (Axelar, LayerZero, Wormhole) solves
reach by inserting a committee, validator set, or verifier quorum into the trust
path. IBC removes that intermediary entirely: the two chains verify each other
directly with on-chain light clients. This page quantifies the value at stake,
the network IBC connects Stellar to, and the concrete difference between
light-client and committee-based interoperability.

{: .note }
> Figures below are dated **July 2026** and sourced from DeFiLlama, Map of Zones,
> the IBC Protocol documentation, and the Stellar Development Foundation's Q1 2026
> report. See [Sources & notes](#sources--notes) for methodology and caveats.

---

## 1. What Stellar brings to the table

Stellar is not a small chain looking for relevance, it is a large settlement
network whose value is currently under-connected.

| Metric | Figure | What it measures |
|---|---|---|
| USDC on Stellar | **~$820M** | Circulating pegged-USD, primarily Circle USDC |
| On-chain RWAs | **$2B+** | Tokenized real-world assets (Ondo, Spiko, WisdomTree), up ~2.5× in one quarter |
| Stablecoin payment volume | **$5.5B** (Q1 2026) | All-time high, +72% YoY |
| DeFi TVL | **~$230M** | Blend (~$140M lending), Aquarius (~$45M DEX), native DEX/AMM |
| DEX volume (30d) | **~$87.5M** | On-chain DEX throughput |
| Network scale | **~10.5M accounts**, 99.99% uptime, ~$0.0001 avg fee | Ten-year-old payments network |

The takeaway: Stellar contributes a deep base of regulated stablecoins,
institutional RWAs, and real payment flow, assets the rest of the interop graph
genuinely wants access to. IBC is the trust-minimized doorway that lets that
value move both ways.

---

## 2. What Stellar gains: the network on the other side

The case for IBC is not "another way to send USDC out." It is **membership in a
live, high-throughput interop network** that Stellar cannot cleanly reach today,
in both directions and without custody risk.

| What IBC connects | Figure |
|---|---|
| Chains reachable | **115+** sovereign chains |
| Value secured | **~$40B / year** |
| Cross-chain volume | **~$0.5–2.7B / 30 days** |
| In production since | **April 2021** |
| Protocol-level exploits | **Zero** |
| Median transfer latency | **~19s** |

Connecting Stellar plugs its stablecoins, RWAs, and payment flow straight into
that network, trust-minimized, and opens the venues on it as places where
Stellar-issued assets can be traded, lent, and used as collateral without
wrapping through multiple bridges first. The same door works inward: assets and
users from 115+ chains reach Stellar's anchor network, cash-out points, and
remittance corridors.

### The compounding part

The number above is today's. The point of joining a protocol rather than
building a bridge is that the number grows without further work on Stellar's
side: **every chain that joins IBC afterwards becomes reachable from Stellar at
near-zero marginal cost**, because the protocol layer, the light client, and the
relayer are all shared. A bridge connects two chains. A protocol implementation
connects Stellar to the graph, including the parts of it that do not exist yet.

### A two-way asset street

Stellar carries assets the interop graph barely has (**non-USD tokenized fiat**
(BRL, ARS, NGN, KES, and more) and **SEP-8 regulated assets**) while the graph
carries assets Stellar users cannot easily obtain today. IBC lets both sides
issue and trade natively, making Stellar a *supplier* of unique assets to the
network, not just another consumer of USDC.

### The flagship use case, stated concretely

IBC is a capability rather than a use case, so here is one use case with a
specific mechanism rather than a list of possibilities.

Stellar is **the only chain carrying tokenized non-USD currency at any meaningful
scale**: naira, reais, pesos and shillings, issued by regional anchors and used
for payments. The rest of the IBC graph is almost entirely dollar-denominated and
holds very little of this.

The significance is a question of **market structure**. A stablecoin that cannot
be traded has limited utility. For a market to function, a participant has to
hold inventory and quote prices, and holding a regional currency means carrying
the risk that it moves against them. Without somewhere to offset that risk,
market makers either quote poor prices or decline to participate, the market
stays thin, issuance stays small, and the anchor cannot grow.

The IBC graph provides exactly what is missing: **deep spot liquidity and
derivatives venues where that risk can be hedged**. Stellar provides the assets.
Neither side can develop the market independently, and no alternative pairing
substitutes, because no other chain issues these currencies.

The counter-argument (that this exports trading activity off Stellar) is answered
directly, including where it holds, on
[Questions & Objections](questions.html#does-this-move-liquidity-off-stellar).

### Beyond token transfer

Token transfer is the first application on the packet layer, not the ceiling, and
IBC v2 makes that unusually cheap to exploit. Under v1 an application was bound
to a channel at handshake time, so every new use case meant new plumbing. V2
carries the source and destination ports **inside the packet payload**, so a
single client pairing already carries packets for any number of applications,
including ones not yet written. Nothing about the trust model changes when a
second application appears; it rides the verification already built.

| Direction | What it enables |
|---|---|
| **ICS-27** interchain accounts | A contract or account on another chain controls an account on Stellar, so a strategy running elsewhere can hold and move Stellar-native assets **without a custodian in between** |
| **ICS-31** cross-chain queries | A Soroban contract reads *verified* state from another chain, which is what lets it price or settle against facts it did not observe locally |
| **Symmetric packet layer** | An application on another chain can trigger Soroban execution and receive the acknowledgement, so Stellar becomes a **callable destination**, not only a source of assets |

The practical consequence for planning is that **the expensive part is paid
once**. The light client, the relayer, the proof plumbing and the operational
work are shared by every application that follows, so the second and third
product on this rail cost a fraction of the first.

---

## 3. Why IBC, when Stellar already has bridges

The sharpest objection is fair: *Stellar can already reach other chains via
Axelar or LayerZero, why build IBC?* Because those protocols do not remove the
trusted intermediary; they relocate it. IBC does.

### The trust models are not equivalent

| Protocol | Who verifies a transfer | Trust assumption |
|---|---|---|
| **IBC** | The destination chain's on-chain **light client** of the source chain | The **two chains' own** consensus assumptions, the counterparty's validator set on one side, Stellar's quorum configuration on the other. Nothing else |
| **Axelar** | Axelar's **proof-of-stake validator set** observes and signs | Honest majority of **Axelar's** validators (a third chain in the middle) |
| **LayerZero** | A configurable **DVN** quorum (defaults: Google Cloud + LayerZero Labs 2-of-3 multisig) | Honest majority of the configured **third-party verifiers** |
| **Chainlink CCIP** | An honest majority of **1 of 3 oracle networks** + Risk Management Network | Honest majority of **Chainlink's** oracle/DON layer |

In every non-IBC row there is an *added* party sitting between the two chains. If
enough of that committee colludes or is compromised, transfers can be forged.
That is not theoretical (**bridges are the most-exploited category in crypto:
$2.5B+ stolen, and in 2022 bridge hacks were ~70% of all crypto theft** (Ronin
$625M, Poly $611M, Wormhole $326M) all committee-in-the-middle designs).

IBC has no such committee. Each chain runs a light client of the other and
verifies its consensus directly, so **the security of a transfer equals the
security of the two chains it connects, nothing weaker is added in between.**

### IBC's track record

| Property | IBC |
|---|---|
| Chains connected | **115+** |
| In production since | **April 2021** (~4 years) |
| Protocol-level exploits | **Zero** |
| Value secured | **~$40B/year** |
| Cross-chain volume | **~$0.5–2.7B / 30 days** (Map of Zones) |
| Median transfer latency | **~19s** (vs LayerZero ~107–298s) |
| Chain coverage vs peers | 115+ chains (IBC) · 83 (LayerZero) · 69 (Axelar) · 9 (CCIP) |
| Integration model | **Permissionless** · no protocol/vendor fees |

The honest one-liner: **with a bridge, you trust the bridge. With IBC, you trust
Stellar and the chain you're talking to, and nothing else.**

### Why Stellar can join now

IBC was historically confined to a single chain family because the protocol was
too heavy to implement anywhere else. **IBC v2 (Eureka)** removed that barrier,
collapsing the old multi-step handshakes into a single counterparty registration
and reaching Ethereum (and soon Solana) at roughly **$0.97 per transfer**. The
**08-wasm** light-client module lets a chain like Stellar plug in by deploying a
light client as WASM bytecode, with no chain-wide upgrade required on the host.
This is the first moment Stellar has been technically eligible to join IBC as a
first-class member, and one of the first outside the original chain family to do
so.

---

## 4. The bottom line

Stellar brings a deep base of regulated stablecoins, institutional RWAs, and real
payment volume. IBC connects that value, trust-minimized, to a live network of
**115+ chains** moving billions per month with zero protocol-level exploits since
2021. Unlike every bridge Stellar could use instead, IBC adds **no new trust
assumption**: no committee, no validator set, no verifier quorum in the middle.

And because IBC is shared infrastructure rather than a point bridge, the
investment compounds: connect Stellar once, and it reaches the entire graph, the
next chain that joins IBC becomes reachable from Stellar at near-zero marginal
cost. For the full architecture of *how* this works, see
[Architecture](architecture.html); for the evidence that the hard part already
works, see [Implementation & Evidence](implementation.html); for the reasoning
behind each design choice, see [Strategy](strategy.html).

The fair objections to all of this (the use case, the liquidity question, whether
IBC fits Stellar at all, what would have to fail for funds to be lost, and the
delivery risk of a small team) are answered on
[Questions & Objections](questions.html).

---

## Sources & notes
{: .no_toc }

Figures captured **July 2026**. Notes on methodology:

- **Stellar network figures** (RWAs, stablecoin payment volume, accounts) are
  from the Stellar Development Foundation's Q1 2026 report; treated as
  company-reported.
- **IBC 30-day volume** is given as a range: the live Map of Zones API's
  "switched" (net) figure (~$499M) versus its published gross figure (~$2.7B).
  Cited the conservative end for defensibility.

### Metrics

- Stellar TVL, DEX volume & protocols, [DeFiLlama: Stellar](https://defillama.com/chain/stellar)
- IBC zones, transfer volume & per-chain activity, [Map of Zones](https://mapofzones.com/)
- Stablecoin circulation, [DeFiLlama Stablecoins](https://stablecoins.llama.fi/stablecoinchains)
- Stellar network stats (accounts, payments, assets), [stellar.expert](https://stellar.expert/explorer/public)
- Stellar RWA / payment volume, [SDF Q1 2026 report](https://stellar.org/blog/foundation-news/q1-2026-execution-at-network-scale)

### Interoperability & IBC track record

- Trust assumptions in interoperability, [ibcprotocol.dev](https://ibcprotocol.dev/blog/trust-assumptions-in-interoperability)
- Interoperability solution comparison, [ibcprotocol.dev](https://ibcprotocol.dev/interoperability-solution-comparison)
- IBC vs LayerZero, [ibcprotocol.dev](https://ibcprotocol.dev/blog/the-interop-series-ibc-and-layerzero)
- IBC vs Chainlink CCIP, [ibcprotocol.dev](https://ibcprotocol.dev/blog/comparative-analysis-dissecting-ibc-and-ccip)
- 12 IBC myths, [ibcprotocol.dev](https://ibcprotocol.dev/blog/12-ibc-myths)
- IBC v2 (Eureka) announcement, [ibcprotocol.dev](https://ibcprotocol.dev/blog/ibc-v2-announcement)
- The 08-wasm light client, [ibcprotocol.dev](https://ibcprotocol.dev/blog/wasm-client)
- What is IBC, [Interchain Foundation](https://medium.com/the-interchain-foundation/what-is-ibc-interchain-stack-highlights-35e309740165)
- IBC Protocol, [ibcprotocol.dev](https://ibcprotocol.dev/)

### Bridge security

- Cross-chain bridge hacks & stolen value, [Chainalysis](https://www.chainalysis.com/blog/cross-chain-bridge-hacks-2022/)
