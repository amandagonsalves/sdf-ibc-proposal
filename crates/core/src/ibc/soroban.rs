use anyhow::{anyhow, Result};
use stellar_xdr::curr::{
    Limits, ScBytes, ScMap, ScMapEntry, ScString, ScSymbol, ScVal, StringM, VecM, WriteXdr,
};

use super::client_state::AnyClientState;
use super::consensus_state::AnyConsensusState;

fn sc_symbol(s: &str) -> Result<ScVal> {
    let m: StringM<32> = s
        .try_into()
        .map_err(|_| anyhow!("invalid struct field symbol: {s}"))?;
    Ok(ScVal::Symbol(ScSymbol(m)))
}

fn sc_string(s: &str) -> Result<ScVal> {
    let m: StringM = s
        .try_into()
        .map_err(|_| anyhow!("invalid string for ScVal"))?;
    Ok(ScVal::String(ScString(m)))
}

fn sc_bytes(b: Vec<u8>) -> Result<ScVal> {
    let bytes: ScBytes = b
        .try_into()
        .map_err(|_| anyhow!("invalid bytes for ScVal"))?;
    Ok(ScVal::Bytes(bytes))
}

fn sc_struct(fields: Vec<(&str, ScVal)>) -> Result<ScVal> {
    let mut entries = Vec::with_capacity(fields.len());
    for (key, val) in fields {
        entries.push(ScMapEntry {
            key: sc_symbol(key)?,
            val,
        });
    }
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    let vm: VecM<ScMapEntry> = entries
        .try_into()
        .map_err(|_| anyhow!("struct map too large"))?;
    Ok(ScVal::Map(Some(ScMap(vm))))
}

fn sc_height(revision_number: u64, revision_height: u64) -> Result<ScVal> {
    sc_struct(vec![
        ("revision_number", ScVal::U64(revision_number)),
        ("revision_height", ScVal::U64(revision_height)),
    ])
}

impl AnyClientState {
    pub fn to_soroban_xdr(&self) -> Result<Vec<u8>> {
        let AnyClientState::Tendermint(cs) = self;
        let cs = cs.inner();

        let trust_level = sc_struct(vec![
            ("numerator", ScVal::U32(cs.trust_level.numerator() as u32)),
            (
                "denominator",
                ScVal::U32(cs.trust_level.denominator() as u32),
            ),
        ])?;

        let (frozen_rn, frozen_rh) = cs
            .frozen_height
            .as_ref()
            .map(|h| (h.revision_number(), h.revision_height()))
            .unwrap_or((0, 0));

        let state = sc_struct(vec![
            ("chain_id", sc_string(cs.chain_id.as_str())?),
            ("trust_level", trust_level),
            (
                "trusting_period_secs",
                ScVal::U64(cs.trusting_period.as_secs()),
            ),
            (
                "unbonding_period_secs",
                ScVal::U64(cs.unbonding_period.as_secs()),
            ),
            (
                "max_clock_drift_secs",
                ScVal::U64(cs.max_clock_drift.as_secs()),
            ),
            (
                "latest_height",
                sc_height(
                    cs.latest_height.revision_number(),
                    cs.latest_height.revision_height(),
                )?,
            ),
            ("is_frozen", ScVal::Bool(cs.frozen_height.is_some())),
            ("frozen_height", sc_height(frozen_rn, frozen_rh)?),
            ("proof_specs", sc_bytes(Vec::new())?),
        ])?;

        state
            .to_xdr(Limits::none())
            .map_err(|e| anyhow!("client_state to_xdr: {e}"))
    }
}

impl AnyConsensusState {
    pub fn to_soroban_xdr(&self) -> Result<Vec<u8>> {
        let AnyConsensusState::Tendermint(cons) = self;
        let cons = cons.inner();

        let timestamp_secs = cons.timestamp.unix_timestamp().max(0) as u64;
        let next_validators_hash = cons.next_validators_hash.as_bytes().to_vec();
        let root = cons.root.as_bytes().to_vec();

        let state = sc_struct(vec![
            ("timestamp_secs", ScVal::U64(timestamp_secs)),
            ("next_validators_hash", sc_bytes(next_validators_hash)?),
            ("root", sc_bytes(root)?),
        ])?;

        state
            .to_xdr(Limits::none())
            .map_err(|e| anyhow!("consensus_state to_xdr: {e}"))
    }
}
