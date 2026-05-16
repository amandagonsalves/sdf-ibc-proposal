extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::{Height, Status};
use ibc_core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core::host::types::error::DecodingError;
use ibc_core::host::types::identifiers::{ClientId, ClientType as IbcClientType};
use ibc_core::host::types::path::{Path, PathBytes};
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::Timestamp;

use crate::CLIENT_TYPE;

pub const TYPE_URL: &str = "/ibc.lightclients.stellar.v1.ClientState";

#[derive(Clone, Debug, PartialEq)]
pub struct StellarClientState {
    pub chain_id: String,
    pub latest_height: Height,
    pub trusting_period_secs: u64,
    pub max_clock_drift_secs: u64,
    pub frozen_height: Option<Height>,
    pub known_validators: Vec<[u8; 32]>,
    pub trust_threshold: (u64, u64),
}

impl StellarClientState {
    pub fn is_frozen(&self) -> bool {
        self.frozen_height.is_some()
    }
}

impl ClientStateCommon for StellarClientState {
    fn verify_consensus_state(
        &self,
        _consensus_state: Any,
        _host_timestamp: &Timestamp,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn client_type(&self) -> IbcClientType {
        IbcClientType::new(CLIENT_TYPE).expect("10-stellar is a valid client type identifier")
    }

    fn latest_height(&self) -> Height {
        self.latest_height
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        if proof_height > self.latest_height {
            return Err(ClientError::InvalidHeight);
        }
        Ok(())
    }

    fn verify_upgrade_client(
        &self,
        _upgraded_client_state: Any,
        _upgraded_consensus_state: Any,
        _proof_upgrade_client: CommitmentProofBytes,
        _proof_upgrade_consensus_state: CommitmentProofBytes,
        _root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        Err(ClientError::ClientSpecific {
            description: "upgrade verification not yet implemented".into(),
        })
    }

    fn serialize_path(&self, path: Path) -> Result<PathBytes, ClientError> {
        Ok(PathBytes::from(path.to_string().into_bytes()))
    }

    fn verify_membership_raw(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: PathBytes,
        _value: Vec<u8>,
    ) -> Result<(), ClientError> {
        Err(ClientError::ClientSpecific {
            description: "membership verification not yet implemented".into(),
        })
    }

    fn verify_non_membership_raw(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: PathBytes,
    ) -> Result<(), ClientError> {
        Err(ClientError::ClientSpecific {
            description: "non-membership verification not yet implemented".into(),
        })
    }
}

impl<V> ClientStateValidation<V> for StellarClientState
where
    V: ClientValidationContext,
    V::ConsensusStateRef: ConsensusStateTrait,
{
    fn verify_client_message(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        _client_message: Any,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn check_for_misbehaviour(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        _client_message: Any,
    ) -> Result<bool, ClientError> {
        Ok(false)
    }

    fn status(&self, _ctx: &V, _client_id: &ClientId) -> Result<Status, ClientError> {
        if self.is_frozen() {
            return Ok(Status::Frozen);
        }
        Ok(Status::Active)
    }

    fn check_substitute(&self, _ctx: &V, _substitute_client_state: Any) -> Result<(), ClientError> {
        Err(ClientError::ClientSpecific {
            description: "client recovery not yet implemented".into(),
        })
    }
}

impl<E> ClientStateExecution<E> for StellarClientState
where
    E: ClientExecutionContext,
    E::ClientStateMut: From<StellarClientState>,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        let _ = (ctx, client_id, consensus_state);
        Ok(())
    }

    fn update_state(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        let _ = (ctx, client_id, header);
        Ok(alloc::vec![self.latest_height])
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        _client_message: Any,
    ) -> Result<(), ClientError> {
        let _ = (ctx, client_id);
        Ok(())
    }

    fn update_state_on_upgrade(
        &self,
        _ctx: &mut E,
        _client_id: &ClientId,
        _upgraded_client_state: Any,
        _upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        Err(ClientError::ClientSpecific {
            description: "upgrade not yet implemented".into(),
        })
    }

    fn update_on_recovery(
        &self,
        _ctx: &mut E,
        _subject_client_id: &ClientId,
        _substitute_client_state: Any,
        _substitute_consensus_state: Any,
    ) -> Result<(), ClientError> {
        Err(ClientError::ClientSpecific {
            description: "recovery not yet implemented".into(),
        })
    }
}

impl TryFrom<Any> for StellarClientState {
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

impl From<StellarClientState> for Any {
    fn from(state: StellarClientState) -> Self {
        let _ = state;
        Any {
            type_url: TYPE_URL.into(),
            value: alloc::vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_client_state() -> StellarClientState {
        StellarClientState {
            chain_id: "stellar-testnet".into(),
            latest_height: Height::new(0, 100).unwrap(),
            trusting_period_secs: 3600,
            max_clock_drift_secs: 60,
            frozen_height: None,
            known_validators: alloc::vec![[0u8; 32]],
            trust_threshold: (2, 3),
        }
    }

    #[test]
    fn client_type_is_stellar() {
        let cs = dummy_client_state();
        assert_eq!(cs.client_type().as_str(), "10-stellar");
    }

    #[test]
    fn latest_height_roundtrip() {
        let cs = dummy_client_state();
        assert_eq!(cs.latest_height(), Height::new(0, 100).unwrap());
    }

    #[test]
    fn active_when_not_frozen() {
        let cs = dummy_client_state();
        assert!(!cs.is_frozen());
    }

    #[test]
    fn frozen_when_height_set() {
        let mut cs = dummy_client_state();
        cs.frozen_height = Some(Height::new(0, 50).unwrap());
        assert!(cs.is_frozen());
    }

    #[test]
    fn type_url_mismatch_returns_error() {
        let raw = Any {
            type_url: "wrong".into(),
            value: alloc::vec![],
        };
        assert!(StellarClientState::try_from(raw).is_err());
    }

    #[test]
    fn into_any_has_correct_type_url() {
        let cs = dummy_client_state();
        let any: Any = cs.into();
        assert_eq!(any.type_url, TYPE_URL);
    }
}
