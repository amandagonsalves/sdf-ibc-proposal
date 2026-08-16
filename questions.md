---
title: Questions & Objections
layout: default
nav_order: 3
description: >-
  The fair objections to building IBC v2 for Stellar, each stated at its
  strongest and answered directly, including where the criticism holds.
---

# Questions & Objections
{: .no_toc }

The objections below are the ones we consider fair. Each is stated at its
strongest rather than in a version that is easy to answer, and where a criticism
holds it is conceded rather than argued around.

## Contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## What is the use case?

The fair version of the criticism is that IBC is a *capability*, not a use case.
That is correct, so a single use case is offered here rather than a list.

Stellar is the only chain carrying **tokenized non-USD currency at any meaningful
scale**: naira, reais, pesos and shillings, issued by regional anchors and used
for payments. The remainder of the IBC graph is almost entirely
dollar-denominated and holds very little of this.

The significance is a question of market structure. A stablecoin that cannot be
traded has limited utility. For a market to function, a participant has to hold
inventory and quote prices, and holding a regional currency means carrying the
risk that it moves against them. Without somewhere to offset that risk, market
makers either quote poor prices or decline to participate, the market stays thin,
issuance stays small, and the anchor cannot grow.

The IBC graph provides what is missing: deep spot liquidity and derivatives
venues where that risk can be hedged. Stellar provides the assets. Neither side
can develop the market independently, and no alternative pairing substitutes,
because no other chain issues these currencies.

That is the argument in full: one use case, a specific mechanism, and an
identifiable class of counterparty to approach.

---

## Does this move liquidity off Stellar?

In part, yes.

The technical answer is that the asset never leaves, because ICS-20 escrows it on
Stellar and mints a claim on the destination chain. That is accurate as far as it
goes. The tokens remain locked in a Soroban contract, and issuance, custody,
reserves and redemption all stay on Stellar. It is also not what the question is
really asking.

What does move is **trading activity**. If a regional-currency pair trades on
another chain's venue, that venue earns the fees and holds the depth. That is a
genuine cost, and there are three reasons it is worth accepting.

**First, the alternative is not that the pair trades on Stellar instead. It is
that it does not trade at all.** Stellar's DEX handles roughly $87M a month
across all pairs, within which regional-currency markets are negligible. This
does not divert an existing liquid market; it enables one that does not currently
exist. If hedging becomes possible, the anchor can issue more, which expands the
Stellar-side business.

**Second, Stellar's economics do not rest on DEX fees.** Roughly $5.5B of
quarterly stablecoin payment volume compares with $87M a month of DEX volume.
Issuance, settlement and the on- and off-ramps are where the value is captured,
and all three remain on Stellar.

**Third, assets already leave through less secure routes.** Committee-based
bridges operate on Stellar today. The question has never been whether Stellar's
assets reach other chains, but whether they do so through a light client or
through an operator set.

{: .note }
> **Where the objection holds.** If the strategic goal is for Stellar to become a
> significant DeFi venue in its own right, exporting trading activity works
> against that. This is a real tension rather than a misunderstanding, and it
> deserves to be discussed directly.

---

## Why not integrate an existing bridge?

Because they address a different problem.

Under a committee bridge, an external validator set observes Stellar and reports
to the destination chain, and the security of the transfer rests on that set.
Under IBC, the destination chain runs a light client of Stellar and verifies the
proof itself, so there is no third party to trust and none to compromise. The
distinction can appear academic until one observes that essentially every large
bridge failure has been a compromise of that intermediate layer rather than a
flaw in either chain.

The stronger argument is **regulatory**. A tokenized-treasury issuer or a
licensed anchor operates under legal opinions governing who may hold the asset. A
committee bridge introduces a custodian that those issuers have not vetted and
frequently cannot accept. For them, the absence of a trusted intermediary is not
a preference but the only structure that clears compliance.

There is also a difference in what Stellar acquires. Integrating a bridge
provides one additional route and a lasting dependency on a commercial operator.
Implementing IBC makes Stellar a chain that more than a hundred existing networks
can connect to without requiring anyone's permission, including ours.

---

## Is IBC a poor fit for Stellar?

A reasonable prior, and the question that has occupied most of this year.

The specification does not presuppose any particular consensus mechanism. It
defines a *client interface*, of which Tendermint is one implementation, and the
`08-wasm` module exists precisely so that a chain can host a light client for a
consensus family that was not anticipated.

The difficulty is nonetheless real, and most of the
[Architecture](architecture.html) page is concerned with it. A Tendermint chain
provides a hash chain in which each header commits to its predecessor, and the
header carries the application state root directly. Stellar's consensus provides
neither. There is no global validator set to count, and the value the network
agrees upon does not commit to the ledger's state. Consequently the client must
reproduce Stellar's own definition of agreement, and then construct the binding
from that agreement to a specific ledger and from that ledger to the state being
proved, none of which a same-family integration has to do.

The conclusion is that this is **harder than a same-family integration but not a
poor fit**. It is a different one, and the difference is now implemented and
verified against live mainnet data.

---

## What evidence is there that the verification works?

It has been executed against the live network, and it has been shown to **reject
invalid input** rather than merely to accept valid input.

The [Implementation & Evidence](implementation.html) page sets out the detail. In
summary: the checker runs the contract's own code against real archived consensus
messages and ledger headers; the two links binding a ledger to contract state
reconstructed correctly for **64 of 64** mainnet ledgers in a checkpoint and **40
of 40** Soroban invocations; and an **18-case suite** takes verified input, alters
exactly one element per case, and confirms that the corresponding check refuses
it.

The second property matters more than the first. A verifier that accepts correct
input may not be doing anything at all. One that demonstrably refuses tampered
signatures, incorrect network identifiers, replayed signers, forged quorum sets
and mismatched headers is performing verification.

---

## What would have to fail for funds to be lost?

Three things, of which **only one is plausible**.

**Forging consensus evidence** would require forging Ed25519 signatures for an
entire quorum of Stellar validators, or finding a hash collision at one of the
bindings in the chain of custody. Neither is a practical concern.

**Compromising the relayer, the gateway or the api** would produce delay or
censorship rather than loss. None of them holds a signing key, none can produce a
proof that verifies, and packets that fail to arrive time out and refund the
sender.

**The plausible failure is misconfiguration of the quorum set the client
trusts.** If that is set incorrectly, the client faithfully follows a different
view of the network while every signature verifies correctly. It cannot be
validated from within a contract, which is why it is validated off-chain using
SDF's own analysis tooling, pinned by fingerprint in the relayer so that no
service can substitute an alternative, and monitored for drift thereafter. It
requires a named owner. This is the principal risk, and it is **organisational
rather than cryptographic**.

---

## Who operates the relayer?

Anyone may, and relaying confers no authority, so the question is economic rather
than architectural. A relayer pays transaction fees and receives no privileged
position. If no one relays, packets stop moving and pending ones refund on
timeout: nothing is lost and nothing becomes stuck.

That said, an assumption that someone will volunteer is not an operations plan. A
production deployment requires at least one relayer under monitoring, and the
archive publication cadence that supplies consensus evidence establishes a
latency floor (roughly five and a half minutes outbound) that product decisions
need to accommodate.

---

## What is the delivery risk?

The commit history shows a single author, so the question is fair and is better
addressed here than discovered later. The working arrangement is less exposed
than that history suggests: one engineer writes the code, and **four reviewers
experienced with Stellar** review it, so the design decisions and the
security-critical paths have had more than one set of eyes on them throughout.

Several further factors reduce the exposure:

- The project is **not being built in isolation**, and has support from Cosmos
  Labs and the Cardano Foundation, whose work the relayer and light-client
  patterns build on.
- The protocol is a **public standard** rather than a private design, so a second
  engineer inherits a specification rather than having to reconstruct someone's
  intent.
- The consensus verifier exists as a **standalone crate** that can be read, run
  and audited independently of the rest of the system.
- The test suite and its negative cases **encode the invariants that matter**, so
  a regression fails visibly.
- The planned **third-party audit** places external review on the
  security-critical paths regardless.
