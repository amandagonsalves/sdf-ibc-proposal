---
title: Value & Comparison
layout: default
nav_order: 2
description: >-
  What Stellar gains from IBC connectivity, the liquidity and assets it
  unlocks from the Cosmos and Cardano ecosystems, and how IBC's
  trust-minimized model compares to the bridges Stellar could use instead.
---

# Value & Comparison
{: .no_toc }

What connecting Stellar to IBC is actually worth — the value it brings *into*
the Stellar ecosystem, and why a trust-minimized IBC path is materially better
than the bridges Stellar could otherwise use.

## Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## The thesis

Stellar already holds enormous transactable value — over **$800M in USDC**,
**$2B+ in on-chain real-world assets**, and billions in monthly stablecoin
payment volume. What it lacks is a way to move that value to other chains
*without handing custody to a trusted intermediary.*

Every bridge available to Stellar today — Axelar, LayerZero, Wormhole — solves
reach by inserting a committee, validator set, or verifier quorum into the trust
path. IBC removes that intermediary entirely: the two chains verify each other
directly with on-chain light clients. This page quantifies the value at stake,
the liquidity IBC unlocks from Cosmos and Cardano, and the concrete difference
between light-client and committee-based interoperability.

{: .note }
> Figures below are dated **July 2026** and sourced from DeFiLlama, Map of Zones,
> the IBC Protocol documentation, and the Stellar Development Foundation's Q1 2026
> report. See [Sources & notes](#sources--notes) for methodology and caveats.

---

## 1. What Stellar brings to the table

Stellar is not a small chain looking for relevance — it is a large settlement
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
institutional RWAs, and real payment flow — assets the rest of the interop graph
genuinely wants access to. IBC is the trust-minimized doorway that lets that
value move both ways.

---

## 2. What Stellar gains — value flowing in

The case for IBC is not "another way to send USDC out." It is **new liquidity,
assets, and users flowing into Stellar** from ecosystems it cannot cleanly reach
today.

### From the Cosmos / IBC graph

IBC is a live, high-throughput economy of 115+ sovereign chains. Selected
30-day activity (Map of Zones):

| Chain | 30d IBC volume | Role |
|---|---|---|
| **Noble** | ~$199M | Canonical USDC issuance hub; #1 IBC chain by volume |
| **Osmosis** | ~$90M (~177K transfers) | Deepest IBC AMM; most-connected chain (110 peers) |
| **dYdX** | ~$90M | Perpetuals / derivatives |
| **Cosmos Hub** | ~$16M | ICS-secured services |
| Injective, Neutron, Celestia, … | — | CosmWasm DeFi, data availability |

Connecting Stellar plugs its stablecoins and RWAs straight into this flow —
trust-minimized, no custody risk — and opens Osmosis, Injective, dYdX, and
Neutron as venues where Stellar's USDC, XLM, and tokenized assets can be traded,
lent, and used as collateral without wrapping through multiple bridges first.

### From Cardano

This project is built with the support of the **Cardano Foundation**, reusing the
only production non-Cosmos IBC stack in the industry. That gives Stellar a
*direct*, trust-minimized path to Cardano's DeFi ecosystem — a chain it has no
clean connection to today:

| Cardano metric | Figure |
|---|---|
| DeFi TVL | **~$78M** |
| Top protocols | Minswap (~$18.6M DEX), Liqwid (~$12.6M lending) |
| DEX volume (30d) | **~$85.5M** |

### A two-way asset street

Stellar carries assets the IBC world barely has — **non-USD tokenized fiat**
(BRL, ARS, NGN, KES, and more) and **SEP-8 regulated assets** — while Cosmos and
Cardano carry assets Stellar users cannot easily obtain. IBC lets both sides
issue and trade natively, making Stellar a *supplier* of unique assets to the
graph, not just another consumer of USDC.

---

## 3. Why IBC, when Stellar already has bridges

The sharpest objection is fair: *Stellar can already reach other chains via
Axelar or LayerZero — why build IBC?* Because those protocols do not remove the
trusted intermediary; they relocate it. IBC does.

### The trust models are not equivalent

| Protocol | Who verifies a transfer | Trust assumption |
|---|---|---|
| **IBC** | The destination chain's on-chain **light client** of the source chain | Honest majority of the **two chains' own** validator sets — nothing else |
| **Axelar** | Axelar's **proof-of-stake validator set** observes and signs | Honest majority of **Axelar's** validators (a third chain in the middle) |
| **LayerZero** | A configurable **DVN** quorum (defaults: Google Cloud + LayerZero Labs 2-of-3 multisig) | Honest majority of the configured **third-party verifiers** |
| **Chainlink CCIP** | An honest majority of **1 of 3 oracle networks** + Risk Management Network | Honest majority of **Chainlink's** oracle/DON layer |

In every non-IBC row there is an *added* party sitting between the two chains. If
enough of that committee colludes or is compromised, transfers can be forged.
That is not theoretical — **bridges are the most-exploited category in crypto:
$2.5B+ stolen, and in 2022 bridge hacks were ~70% of all crypto theft** (Ronin
$625M, Poly $611M, Wormhole $326M — all committee-in-the-middle designs).

IBC has no such committee. Each chain runs a light client of the other and
verifies its consensus directly, so **the security of a transfer equals the
security of the two chains it connects — nothing weaker is added in between.**

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
Stellar and the chain you're talking to — and nothing else.**

### Why Stellar can join now

IBC was historically Cosmos-only because the protocol was too heavy to implement
elsewhere. **IBC v2 (Eureka)** removed that barrier — collapsing the old
multi-step handshakes into a single counterparty registration, and reaching
Ethereum (and soon Solana) at roughly **$0.97 per transfer**. The **08-wasm**
light-client module lets a non-Cosmos chain like Stellar plug in by deploying a
light client as WASM bytecode, with no chain-wide upgrade required. This is the
first moment Stellar has been technically eligible to join IBC as a first-class
member — and one of the first non-Cosmos chains to do so.

---

## 4. The bottom line

Stellar brings a deep base of regulated stablecoins, institutional RWAs, and real
payment volume. IBC connects that value — trust-minimized — to a live economy of
115+ chains moving billions per month, plus a direct path to Cardano. Unlike
every bridge Stellar could use instead, IBC adds **no new trust assumption**: no
committee, no validator set, no verifier quorum in the middle.

And because IBC is shared infrastructure rather than a point bridge, the
investment compounds: connect Stellar once, and it reaches the entire graph — the
next chain that joins IBC becomes reachable from Stellar at near-zero marginal
cost. For the full architecture of *how* this works, see
[Architecture](architecture.html); for the reasoning behind each design choice,
see [Strategy](strategy.html).

---

## Sources & notes
{: .no_toc }

Figures captured **July 2026**. Notes on methodology:

- **Stellar network figures** (RWAs, stablecoin payment volume, accounts) are
  from the Stellar Development Foundation's Q1 2026 report; treat as
  company-reported.
- **IBC 30-day volume** is given as a range: the live Map of Zones API's
  "switched" (net) figure (~$499M) versus its published gross figure (~$2.7B).
  Cite the conservative end for defensibility.

### Metrics

- Stellar TVL, DEX volume & protocols — [DeFiLlama: Stellar](https://defillama.com/chain/stellar)
- Cardano TVL, DEX volume & protocols — [DeFiLlama: Cardano](https://defillama.com/chain/cardano)
- Cosmos ecosystem TVL — [DeFiLlama: Cosmos](https://defillama.com/chain/cosmos)
- Stablecoin circulation (USDC on Stellar, Noble) — [DeFiLlama Stablecoins](https://stablecoins.llama.fi/stablecoinchains)
- IBC zones, transfer volume & per-chain activity — [Map of Zones](https://mapofzones.com/)
- Stellar network stats (accounts, payments, assets) — [stellar.expert](https://stellar.expert/explorer/public)
- Stellar RWA / payment volume — [SDF Q1 2026 report](https://stellar.org/blog/foundation-news/q1-2026-execution-at-network-scale)

### Interoperability & IBC track record

- Trust assumptions in interoperability — [ibcprotocol.dev](https://ibcprotocol.dev/blog/trust-assumptions-in-interoperability)
- Interoperability solution comparison — [ibcprotocol.dev](https://ibcprotocol.dev/interoperability-solution-comparison)
- IBC vs LayerZero — [ibcprotocol.dev](https://ibcprotocol.dev/blog/the-interop-series-ibc-and-layerzero)
- IBC vs Chainlink CCIP — [ibcprotocol.dev](https://ibcprotocol.dev/blog/comparative-analysis-dissecting-ibc-and-ccip)
- 12 IBC myths — [ibcprotocol.dev](https://ibcprotocol.dev/blog/12-ibc-myths)
- IBC v2 (Eureka) announcement — [ibcprotocol.dev](https://ibcprotocol.dev/blog/ibc-v2-announcement)
- The 08-wasm light client — [ibcprotocol.dev](https://ibcprotocol.dev/blog/wasm-client)
- What is IBC — [Interchain Foundation](https://medium.com/the-interchain-foundation/what-is-ibc-interchain-stack-highlights-35e309740165)
- IBC Protocol — [ibcprotocol.dev](https://ibcprotocol.dev/)

### Bridge security

- Cross-chain bridge hacks & stolen value — [Chainalysis](https://www.chainalysis.com/blog/cross-chain-bridge-hacks-2022/)
