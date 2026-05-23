use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
};
use prost::Message;

use crate::error::ContractError;
use crate::msg::{
    CheckForMisbehaviourMsg, CheckForMisbehaviourResult, ClientStatus, Height as MsgHeight,
    InstantiateMsg, LatestHeightResult, QueryMsg, StatusResult, SudoMsg, TimestampAtHeightResult,
    UpdateStateMsg, UpdateStateOnMisbehaviourMsg, UpdateStateResult, VerifyMembershipMsg,
    VerifyNonMembershipMsg,
};
use crate::state::{CLIENT_STATE, CONSENSUS_STATES};
use crate::types::{ClientState, ConsensusState, Height as WireHeight, StellarHeader};

#[entry_point]
pub fn instantiate(
    deps: DepsMut<'_>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if CLIENT_STATE.may_load(deps.storage)?.is_some() {
        return Err(ContractError::AlreadyInitialised);
    }

    let client_state = ClientState::decode(msg.client_state.as_slice())
        .map_err(|e| ContractError::InvalidWire(format!("client_state: {e}")))?;
    let consensus_state = ConsensusState::decode(msg.consensus_state.as_slice())
        .map_err(|e| ContractError::InvalidWire(format!("consensus_state: {e}")))?;

    let height = client_state
        .latest_height
        .as_ref()
        .ok_or_else(|| ContractError::InvalidWire("client_state.latest_height".into()))?
        .revision_height;

    CLIENT_STATE.save(deps.storage, &client_state)?;
    CONSENSUS_STATES.save(deps.storage, height, &consensus_state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let data = match msg {
        SudoMsg::UpdateState(m) => to_json(&update_state(deps, env, m)?)?,
        SudoMsg::UpdateStateOnMisbehaviour(m) => {
            update_state_on_misbehaviour(deps, env, m)?;
            Binary::default()
        }
        SudoMsg::CheckForMisbehaviour(m) => to_json(&check_for_misbehaviour(deps, env, m)?)?,
        SudoMsg::VerifyMembership(m) => {
            verify_membership(deps, env, m)?;
            Binary::default()
        }
        SudoMsg::VerifyNonMembership(m) => {
            verify_non_membership(deps, env, m)?;
            Binary::default()
        }
    };
    Ok(Response::default().set_data(data))
}

#[entry_point]
pub fn query(deps: Deps<'_>, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::ClientState {} => {
            let cs = require_client_state(deps)?;
            Ok(Binary::new(cs.encode_to_vec()))
        }
        QueryMsg::ConsensusState { height } => {
            let cons = require_consensus_state(deps, height.revision_height)?;
            Ok(Binary::new(cons.encode_to_vec()))
        }
        QueryMsg::LatestHeight {} => {
            let cs = require_client_state(deps)?;
            let h = cs.latest_height.unwrap_or_default();
            to_json(&LatestHeightResult {
                height: MsgHeight {
                    revision_number: h.revision_number,
                    revision_height: h.revision_height,
                },
            })
        }
        QueryMsg::Status {} => {
            let cs = require_client_state(deps)?;
            let status = if cs.frozen_height.is_some() {
                ClientStatus::Frozen
            } else {
                ClientStatus::Active
            };
            to_json(&StatusResult { status })
        }
        QueryMsg::TimestampAtHeight { height } => {
            let cons = require_consensus_state(deps, height.revision_height)?;
            to_json(&TimestampAtHeightResult {
                timestamp: cons.timestamp,
            })
        }
    }
}

fn update_state(
    deps: DepsMut<'_>,
    _env: Env,
    msg: UpdateStateMsg,
) -> Result<UpdateStateResult, ContractError> {
    let mut cs = require_client_state_mut(deps.as_ref())?;
    if let Some(h) = cs.frozen_height.as_ref() {
        return Err(ContractError::Frozen {
            height: h.revision_height,
        });
    }

    let header = decode_header(&msg.client_message)?;
    let trusted_height = header
        .trusted_height
        .as_ref()
        .ok_or_else(|| ContractError::InvalidWire("header.trusted_height".into()))?
        .revision_height;
    if header.ledger_seq <= trusted_height {
        return Err(ContractError::NonAdvancingHeight {
            trusted: trusted_height,
            target: header.ledger_seq,
        });
    }

    let trusted_consensus = require_consensus_state(deps.as_ref(), trusted_height)?;
    if !header.previous_ledger_hash.is_empty()
        && header.previous_ledger_hash != trusted_consensus.ledger_hash
    {
        return Err(ContractError::LedgerHashChainBroken {
            trusted_hex: hex_encode(&trusted_consensus.ledger_hash),
            header_hex: hex_encode(&header.previous_ledger_hash),
        });
    }


    let new_consensus = ConsensusState {
        timestamp: header.timestamp,
        ledger_hash: header.ledger_hash.clone(),
        root: header.ibc_state_root.clone(),
    };

    if let Some(existing) = CONSENSUS_STATES.may_load(deps.storage, header.ledger_seq)? {
        if existing != new_consensus {
            return Err(ContractError::ConsensusStateConflict {
                height: header.ledger_seq,
            });
        }
    }

    CONSENSUS_STATES.save(deps.storage, header.ledger_seq, &new_consensus)?;

    if header.ledger_seq
        > cs.latest_height
            .as_ref()
            .map(|h| h.revision_height)
            .unwrap_or(0)
    {
        cs.latest_height = Some(WireHeight {
            revision_number: 0,
            revision_height: header.ledger_seq,
        });
        CLIENT_STATE.save(deps.storage, &cs)?;
    }

    Ok(UpdateStateResult {
        heights: vec![MsgHeight {
            revision_number: 0,
            revision_height: header.ledger_seq,
        }],
    })
}

fn update_state_on_misbehaviour(
    deps: DepsMut<'_>,
    _env: Env,
    _msg: UpdateStateOnMisbehaviourMsg,
) -> Result<(), ContractError> {
    let mut cs = require_client_state_mut(deps.as_ref())?;
    let latest = cs.latest_height.clone().unwrap_or_default();
    cs.frozen_height = Some(latest);
    CLIENT_STATE.save(deps.storage, &cs)?;
    Ok(())
}

fn check_for_misbehaviour(
    deps: DepsMut<'_>,
    _env: Env,
    msg: CheckForMisbehaviourMsg,
) -> Result<CheckForMisbehaviourResult, ContractError> {
    let cs = require_client_state_mut(deps.as_ref())?;
    if cs.frozen_height.is_some() {
        return Ok(CheckForMisbehaviourResult {
            found_misbehaviour: false,
        });
    }
    let header = decode_header(&msg.client_message)?;
    if let Some(existing) = CONSENSUS_STATES.may_load(deps.storage, header.ledger_seq)? {
        let header_consensus = ConsensusState {
            timestamp: header.timestamp,
            ledger_hash: header.ledger_hash.clone(),
            root: header.ibc_state_root.clone(),
        };
        return Ok(CheckForMisbehaviourResult {
            found_misbehaviour: existing != header_consensus,
        });
    }
    Ok(CheckForMisbehaviourResult {
        found_misbehaviour: false,
    })
}

fn verify_membership(
    deps: DepsMut<'_>,
    _env: Env,
    msg: VerifyMembershipMsg,
) -> Result<(), ContractError> {
    let cs = require_client_state_mut(deps.as_ref())?;
    if cs.frozen_height.is_some() {
        return Err(ContractError::Frozen {
            height: cs.frozen_height.as_ref().unwrap().revision_height,
        });
    }
    let _consensus = require_consensus_state(deps.as_ref(), msg.height.revision_height)?;

    let _ = msg.path;
    let _ = msg.value;
    Ok(())
}

fn verify_non_membership(
    deps: DepsMut<'_>,
    _env: Env,
    msg: VerifyNonMembershipMsg,
) -> Result<(), ContractError> {
    let cs = require_client_state_mut(deps.as_ref())?;
    if cs.frozen_height.is_some() {
        return Err(ContractError::Frozen {
            height: cs.frozen_height.as_ref().unwrap().revision_height,
        });
    }
    let _consensus = require_consensus_state(deps.as_ref(), msg.height.revision_height)?;

    let _ = msg.path;
    Ok(())
}

fn decode_header(bytes: &[u8]) -> Result<StellarHeader, ContractError> {
    StellarHeader::decode(bytes).map_err(|e| ContractError::InvalidWire(format!("header: {e}")))
}

fn require_client_state(deps: Deps<'_>) -> Result<ClientState, ContractError> {
    CLIENT_STATE
        .may_load(deps.storage)?
        .ok_or(ContractError::NotInitialised)
}

fn require_client_state_mut(deps: Deps<'_>) -> Result<ClientState, ContractError> {
    require_client_state(deps)
}

fn require_consensus_state(
    deps: Deps<'_>,
    height: u64,
) -> Result<ConsensusState, ContractError> {
    CONSENSUS_STATES
        .may_load(deps.storage, height)?
        .ok_or(ContractError::ConsensusStateMissing { height })
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<Binary, ContractError> {
    to_json_binary(value).map_err(|e| ContractError::Std(StdError::generic_err(e.to_string())))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
