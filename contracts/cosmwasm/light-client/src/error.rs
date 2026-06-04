use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("client state already initialised")]
    AlreadyInitialised,

    #[error("client state not initialised")]
    NotInitialised,

    #[error("client is frozen at height {height}")]
    Frozen { height: u64 },

    #[error("invalid wire bytes: {0}")]
    InvalidWire(String),

    #[error("consensus state missing at height {height}")]
    ConsensusStateMissing { height: u64 },

    #[error("header chain_id {got} does not match client chain_id {expected}")]
    ChainIdMismatch { expected: String, got: String },

    #[error("header height {target} must be greater than trusted height {trusted}")]
    NonAdvancingHeight { trusted: u64, target: u64 },

    #[error("conflicting consensus state already stored at height {height}")]
    ConsensusStateConflict { height: u64 },

    #[error(
        "scp quorum not met (envelopes={envelopes}, matched_trusted={matched}, raw_ok={raw_ok}, hash_ok={hash_ok}, signer={signer}, network_id={network_id}, statement_len={statement_len})"
    )]
    QuorumNotMet {
        envelopes: usize,
        matched: usize,
        raw_ok: bool,
        hash_ok: bool,
        signer: String,
        network_id: String,
        statement_len: usize,
    },

    #[error("scp network_id is not configured on the client state")]
    NetworkIdMissing,

    #[error("scp signature verification error: {0}")]
    ScpSignatureError(String),

    #[error("merkle proof verification failed")]
    MerkleVerificationFailed,

    #[error(
        "merkle membership mismatch (key_match={key_match}, value_match={value_match}, siblings={siblings}, height={height}, req_key={req_key}, proof_key={proof_key}, value_len={value_len}/{proof_value_len}, stored_root={stored_root}, computed_root={computed_root})"
    )]
    MembershipMismatch {
        key_match: bool,
        value_match: bool,
        siblings: usize,
        height: u64,
        req_key: String,
        proof_key: String,
        value_len: usize,
        proof_value_len: usize,
        stored_root: String,
        computed_root: String,
    },

    #[error("unknown sudo message variant")]
    UnknownSudo,
}
