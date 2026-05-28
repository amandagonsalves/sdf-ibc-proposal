# Strategy

Why this project exists, what we're building, and the reasoning behind each
architectural choice. Written for stakeholders, contributors, and anyone
evaluating the Cardano–Stellar IBC bridge.

---

## TL;DR

We are building a **trust-minimized bridge between Stellar and Cosmos** using
**IBC v2 (Eureka)**, the most battle-tested cross-chain protocol in the
industry. We are doing it **in partnership with the Cardano Foundation**
because their existing Cardano↔Cosmos IBC infrastructure (Hermes fork, light
clients, orchestration tooling) gives Stellar a working bridge in months
instead of years, and gives Cardano a second non-Cosmos chain validating the
same trust-minimized interop stack.

The result is that **Stellar gains first-class connectivity to every
IBC-enabled chain** — Osmosis, Noble, Neutron, Injective, Cosmos Hub, dYdX,
Celestia, and the rest of the IBC graph — without relying on multisig
committees or federated validators.

---

## 1. Why connect Stellar to Cosmos

Stellar and Cosmos solve **different problems** and combine into something
more valuable than either alone.

**Stellar's strengths:**
- Sub-5-second settlement, sub-cent fees.
- A 10-year-old payments network with a global stablecoin and anchor
  ecosystem (USDC, EURC, MoneyGram, Circle, regional fiat anchors).
- The Stellar Development Foundation, AID-tech rails, and an embedded
  user base focused on remittances, FX, and tokenized real-world assets.
- Soroban smart contracts (Rust/WASM) — the platform now supports the
  general-purpose programmability needed for IBC.

**Cosmos's strengths:**
- The largest collection of **app-specific sovereign chains** in crypto:
  Osmosis (DEX), Injective (derivatives), dYdX (perps), Noble (USDC issuance),
  Celestia (data availability), Neutron (smart contracts), Cosmos Hub
  (ICS-secured services).
- A single shared interop protocol (IBC) connecting all of them.
- Mature on-chain governance, staking, and DeFi primitives.

**What the bridge unlocks for users:**
- **Stellar-native assets reach Cosmos DEXs** — USDC, EURC, XLM, and
  anchored RWAs become tradeable on Osmosis and Injective.
- **Cosmos-native assets reach Stellar payment rails** — ATOM, OSMO, INJ,
  and Noble-issued stablecoins become spendable through Stellar's anchor
  network, MoneyGram cash-out points, and global remittance corridors.
- **Cross-chain settlement** — a remittance app can quote rates on Osmosis,
  settle on Stellar, and pay out at a physical cash agent.
- **Soroban contracts callable from Cosmos** — DeFi composability across
  consensus layers.

Stellar is exceptional at the **on-ramp / off-ramp / payment** layer.
Cosmos is exceptional at the **DeFi / app-chain** layer. Connecting them
turns each ecosystem into a distribution channel for the other.

---

## 2. Why IBC

There are at least a dozen cross-chain protocols (LayerZero, Wormhole,
Axelar, Hyperlane, Chainlink CCIP, deBridge, Synapse, Connext, …). We chose
**IBC** because it is the only one that is simultaneously:

**Trust-minimized.** IBC packet verification is performed by an **on-chain
light client** of the source chain running inside the destination chain.
There is no validator committee, no signing federation, no off-chain
multisig. The security of an IBC packet equals the security of the underlying
chains — nothing weaker.

Most "bridges" today rely on a permissioned set of signers who attest to
events. When that committee is compromised, funds are lost. The five largest
bridge hacks in crypto history (Ronin, Poly, Wormhole, Nomad, Harmony) all
share this pattern. IBC's light-client model makes that class of failure
impossible.

**Battle-tested.** IBC has moved hundreds of billions of dollars cumulative
volume across the Cosmos ecosystem since 2021 with no consensus-level
exploit. It is the most-used cross-chain protocol by transaction count.

**Standard.** IBC is a public, open spec (the
[ibc-protocol.org](https://ibc-protocol.org/) standards). Implementations
exist in Go, Rust, Solidity, and Move. Adopting IBC means Stellar can talk
to *any* IBC-enabled chain — present or future — without bespoke per-pair
integration work. Network effect compounds.

**Aligned with Stellar's values.** Stellar was founded on a "trust anchor"
model: trust is explicit, verifiable, and revocable. IBC's light-client
model is the cross-chain expression of that same principle. It is the
ideologically correct choice for a chain whose founding ethos rejects
opaque intermediaries.

**Composable upgrade path.** Through **ICS-20** (transfer) we get fungible
token transfers; through **ICS-27** (interchain accounts) we get
cross-chain smart-contract calls; through **08-wasm** we get pluggable
light clients without forking the counterparty chain. The same primitive
extends from "move USDC" to "trigger a Soroban contract from Osmosis."

---

## 3. Why work with Cardano

The Cardano Foundation has been building **the only production
non-Cosmos IBC integration in the industry** since 2023. They've shipped:

- A **`hermes-relayer` fork** with an abstract `ChainEndpoint` trait
  capable of relaying to non-Tendermint chains. This is the single
  hardest piece of work in any IBC extension project; we get to reuse
  the architectural pattern wholesale.
- A **`cardano-entrypoint`** Cosmos chain (ibc-go v10 + 08-wasm
  enabled) that serves as a reference Cosmos counterparty — we test
  against the same chain Cardano uses, which means our 08-wasm uploads,
  light-client lifecycle, and packet flows are validated against
  known-good Cosmos infrastructure.
- A **`caribic` CLI** orchestrating Docker-based devnets, contract
  deploys, and bridge bootstrap flows. Our own `crates/osmosis` and
  `ci/flows/` are direct descendants of patterns proven there.
- An **08-cardano-probabilistic light client** — a wasm light client for
  a non-Tendermint chain. The architectural template (probabilistic
  finality, snapshot verification, SCP-style consensus modeling) maps
  cleanly to Stellar's SCP.

**This collaboration is uniquely valuable** for three reasons:

1. **Time to market.** Building the relayer fork, light-client crate,
   and orchestration tooling from scratch would take 18–24 months.
   Building on Cardano's foundations compresses it to months.
2. **Cross-pollination of architecture.** Both Cardano and Stellar are
   non-Tendermint chains with their own consensus families (Ouroboros
   PoS / SCP). Patterns that work for one inform the other. Cardano's
   probabilistic light client teaches us how to handle "soft finality"
   for cross-chain proofs; Stellar's faster (5s) finality validates
   simpler client designs.
3. **Compounding investment.** Every improvement to the shared
   `hermes-relayer` fork benefits both ecosystems. Cardano gets
   confirmation that its IBC stack generalizes; Stellar gets a
   maintained relayer without the burden of independent forks.

The Cardano Foundation also brings institutional credibility, security
audits, and a track record of shipping interoperability work — material
when our bridge holds bridged assets at scale.

---

## 4. Why Hermes relayer

Hermes is the **reference Rust IBC relayer** maintained by Informal
Systems (the team behind Tendermint Core). We chose it over the
alternatives — Go relayer (`rly`), Confio's ts-relayer, custom code —
because:

**Maturity.** Hermes has been the workhorse of Cosmos IBC since 2021. It
runs in production relaying significant TVL daily. Bug surface is well
understood; failure modes are documented.

**Rust-native.** The rest of our stack is Rust (Soroban contracts,
`stellar-ibc-core`, gateway, api, light-client-wasm). Hermes lets us
contribute upstream and debug across the entire stack in a single
language and toolchain. No FFI, no Go↔Rust impedance mismatch.

**Forkable, extensible architecture.** Hermes splits chain logic behind
a `ChainEndpoint` trait. Cardano's fork added `CardanoChainEndpoint`;
we add `StellarChainEndpoint` the same way. The fork pattern is proven
and the codebase invites it. Rolling our own relayer would mean
re-implementing event subscription, transaction queuing, light-client
update logic, packet timeouts, fee estimation, key management, and
configuration — all of which Hermes solves.

**IBC v2 support.** Recent Hermes versions support the v2 packet
lifecycle natively. Less code to write, less risk of spec deviation.

**Operator ecosystem.** Cosmos relayer operators already know how to
run Hermes. When the bridge goes live, the operator-facing surface
(`hermes start`, `~/.hermes/config.toml`, `keys add`, `query packet
…`) is familiar.

---

## 5. Why IBC v2 instead of v1

IBC v2 (Eureka) ships a dramatically simpler protocol than v1, and we
benefit on every axis.

**No handshakes.** V1 requires a multi-step `Connection*` + `Channel*`
handshake to establish a route between two chains — four messages per
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
cost-per-packet for application chains. It also simplifies the light
client: 3 path patterns to verify instead of 8.

**Per-packet app routing.** V1 binds an application (port) to a channel
at handshake time. V2 puts `sourcePort` / `destPort` into the packet
payload itself. A single client connection can carry packets for any
number of applications, including future apps we haven't deployed yet.

**Cheaper, faster, simpler client lifecycle.** No `ChannelOpenInit/Try/
Ack/Confirm`, no version negotiation, no port binding. The
`StellarGatewayQuery` service in this repo doesn't even implement
`QueryClientState`, `QueryConsensusState`, or `QueryNextSeqRecv` — all
three are non-provable in v2.

**Greenfield advantage.** V2 is the right call for a chain with no v1
legacy. We pay no migration tax, no backward-compatibility cost. The
ecosystem direction is clearly v2: Eureka chains (dYdX v4, Noble,
Skip's chain abstraction stack) are v2-first. Cosmos Hub is migrating.
Adopting v2 means Stellar plugs into the *current generation* of the
IBC graph, not the legacy one.

**Smaller attack surface.** Less protocol surface = less to audit, less
to get wrong, less for adversaries to probe.

---

## 6. Why this is beneficial for Stellar and Cardano

### For Stellar

**Immediate liquidity reach.** The day the bridge goes live, every
IBC-enabled chain becomes a destination for Stellar's stablecoins
(USDC, EURC) and native assets (XLM). The reverse is also true:
ATOM, OSMO, INJ, dYdX, TIA, NOBLE, and every IBC-graph asset can flow
into Stellar's payment network.

**Validation of Soroban as a serious smart-contract platform.**
Implementing a non-trivial cross-chain protocol on Soroban —
deterministic SMT, ICS-23 proof verification, light-client contracts
— demonstrates that Soroban is production-ready for systems work, not
just simple token logic.

**Differentiator vs other L1 payment networks.** Most payment-focused
chains (Ripple, Algorand, Hedera) have either bespoke bridges or rely
on federated message buses. Stellar with IBC becomes the *only*
trust-minimized payment chain plugged into the largest interop graph
in crypto.

**Real-world asset (RWA) corridor.** Stellar anchors tokenize fiat,
gold, real estate. IBC lets those tokenized RWAs reach Cosmos DeFi —
margining, lending, derivatives — without leaving a trust-minimized
custody path.

**No moat erosion.** Stellar's existing strengths (fast/cheap
settlement, anchor network, regulated stablecoins) are amplified by
IBC, not replaced. Cosmos chains gain access to Stellar's payment
rails; Stellar gains access to Cosmos liquidity. Each chain stays
focused on what it does best.

**Soroban contract callable from another chain.** ICS-27 interchain
accounts and ICS-31 cross-chain queries (both v2-compatible) let a
contract on Osmosis trigger Soroban execution. Stellar becomes a
*destination for cross-chain logic*, not just an end-point.

### For Cardano

**Cross-chain stack validation.** Cardano Foundation's `hermes-relayer`
fork, light-client patterns, and `caribic` tooling now support **two**
non-Tendermint chain families. Pattern reuse is the strongest evidence
that an architecture generalizes. Future non-Cosmos integrations
(Polkadot, Bitcoin via the same probabilistic LC pattern, etc.) become
easier to argue for and execute.

**Shared maintenance burden.** Two teams working on the same Hermes
fork yields more fixes, more features, better test coverage, faster
upstream merges. Cardano's investment compounds.

**Strategic position as the trust-minimized-interop reference
implementer.** The Cardano Foundation has been first-to-ship in
non-Cosmos IBC. Adding Stellar establishes a multi-chain footprint
that meaningfully differentiates Cardano from L1s with only federated
bridges. "If you want a non-Cosmos chain integrated into IBC, the
Cardano Foundation has done it twice."

**Cross-pollination of Cardano and Stellar.** Cardano's Plutus
contracts and Stellar's Soroban contracts can now exchange value
through the Cosmos IBC graph — a Cardano-Stellar path through Osmosis,
for example. Indirect connectivity becomes direct in protocol, even
where no direct bilateral bridge exists.

**Funding-eligible flagship.** Multi-chain bridge work funded by the
Cardano Foundation that ships into another major L1 ecosystem is
exactly the kind of high-leverage interop work Cardano's Catalyst,
Treasury, and partnership programs are designed to support. The work
itself becomes a case study.

---

## In one paragraph

Stellar's payment rails plus Cosmos's app-chain ecosystem is a
combination no current bridge serves trust-minimally. IBC v2 is the
right protocol — battle-tested, standard, light-client-secured, and
small enough to deploy cost-effectively on Soroban. Cardano's
production-grade non-Cosmos IBC stack is the right foundation —
reusing it cuts years off the project, and the partnership compounds
into a multi-chain interop platform that benefits both ecosystems
permanently. Hermes is the right relayer because it is the reference
implementation that already supports this pattern. We don't have to
invent the protocol, the relayer, or the orchestration. We have to
implement the Stellar-shaped piece, and that is exactly what this
repository does.
