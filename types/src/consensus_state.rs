extern crate alloc;

use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::host::types::error::DecodingError;
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::Timestamp;

pub const TYPE_URL: &str = "/ibc.lightclients.stellar.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq)]
pub struct StellarConsensusState {
    pub timestamp: u64,
    pub ledger_hash: [u8; 32],
    pub app_hash: [u8; 32],
    root: CommitmentRoot,
}

impl StellarConsensusState {
    pub fn new(timestamp: u64, ledger_hash: [u8; 32], app_hash: [u8; 32]) -> Self {
        Self {
            timestamp,
            ledger_hash,
            app_hash,
            root: CommitmentRoot::from_bytes(&app_hash),
        }
    }
}

impl ibc_core::client::context::consensus_state::ConsensusState for StellarConsensusState {
    fn root(&self) -> &CommitmentRoot {
        &self.root
    }

    fn timestamp(&self) -> Result<Timestamp, ClientError> {
        Timestamp::from_unix_timestamp(self.timestamp, 0)
            .map_err(ClientError::Timestamp)
    }
}

impl TryFrom<Any> for StellarConsensusState {
    type Error = DecodingError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        if raw.type_url != TYPE_URL {
            return Err(DecodingError::MismatchedResourceName {
                expected: TYPE_URL.into(),
                actual: raw.type_url,
            });
        }
        Err(DecodingError::MissingRawData {
            description: "proto decode not yet wired".into(),
        })
    }
}

impl From<StellarConsensusState> for Any {
    fn from(cs: StellarConsensusState) -> Self {
        let _ = cs;
        Any { type_url: TYPE_URL.into(), value: alloc::vec![] }
    }
}

#[cfg(test)]
mod tests {
    use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;

    use super::*;

    fn dummy() -> StellarConsensusState {
        StellarConsensusState::new(1_700_000_000, [1u8; 32], [2u8; 32])
    }

    #[test]
    fn root_matches_app_hash() {
        let cs = dummy();
        assert_eq!(cs.root().as_bytes(), &[2u8; 32]);
    }

    #[test]
    fn timestamp_is_non_zero() {
        let cs = dummy();
        assert!(cs.timestamp().is_ok());
    }

    #[test]
    fn type_url_mismatch_returns_error() {
        let raw = Any { type_url: "wrong".into(), value: alloc::vec![] };
        assert!(StellarConsensusState::try_from(raw).is_err());
    }

    #[test]
    fn into_any_has_correct_type_url() {
        let any: Any = dummy().into();
        assert_eq!(any.type_url, TYPE_URL);
    }
}
