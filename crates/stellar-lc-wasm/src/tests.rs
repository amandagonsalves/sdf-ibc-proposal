use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::Binary;
use prost::Message;

use crate::entrypoint::{instantiate, query, sudo};
use crate::error::ContractError;
use crate::msg::{
    CheckForMisbehaviourMsg, CheckForMisbehaviourResult, ClientStatus, Height as MsgHeight,
    InstantiateMsg, LatestHeightResult, QueryMsg, StatusResult, SudoMsg, TimestampAtHeightResult,
    UpdateStateMsg, UpdateStateOnMisbehaviourMsg, UpdateStateResult, VerifyMembershipMsg,
    VerifyNonMembershipMsg,
};
use crate::types::{ClientState, ConsensusState, Height as WireHeight, ScpEnvelope, StellarHeader};

const CHAIN_ID: &str = "stellar-testnet";
const ROOT_INIT: [u8; 32] = [0x11; 32];
const ROOT_NEXT: [u8; 32] = [0x22; 32];
const LEDGER_HASH_INIT: [u8; 32] = [0xaa; 32];
const LEDGER_HASH_NEXT: [u8; 32] = [0xbb; 32];

fn fresh_client_state(latest_height: u64) -> ClientState {
    ClientState {
        chain_id: CHAIN_ID.to_string(),
        latest_height: Some(WireHeight {
            revision_number: 0,
            revision_height: latest_height,
        }),
        frozen_height: None,
        trusted_validators: vec![vec![0x42; 32]],
        proof_specs: vec![],
    }
}

fn fresh_consensus_state(ts: u64, ledger_hash: [u8; 32], root: [u8; 32]) -> ConsensusState {
    ConsensusState {
        timestamp: ts,
        ledger_hash: ledger_hash.to_vec(),
        root: root.to_vec(),
    }
}

fn encode<T: Message>(m: &T) -> Binary {
    Binary::new(m.encode_to_vec())
}

fn header(
    trusted_height: u64,
    target_height: u64,
    ts: u64,
    previous_ledger_hash: [u8; 32],
    ledger_hash: [u8; 32],
    root: [u8; 32],
) -> StellarHeader {
    StellarHeader {
        ledger_seq: target_height,
        ledger_header_xdr: vec![],
        ibc_state_root: root.to_vec(),
        scp_envelopes: vec![ScpEnvelope {
            node_id: vec![0x42; 32],
            statement_xdr: vec![1, 2, 3],
            signature: vec![0u8; 64],
        }],
        trusted_height: Some(WireHeight {
            revision_number: 0,
            revision_height: trusted_height,
        }),
        timestamp: ts,
        ledger_hash: ledger_hash.to_vec(),
        previous_ledger_hash: previous_ledger_hash.to_vec(),
    }
}

fn do_instantiate(deps: &mut cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
>) {
    let env = mock_env();
    let info = message_info(&deps.api.addr_make("creator"), &[]);
    let cs = fresh_client_state(100);
    let cons = fresh_consensus_state(1_000_000, LEDGER_HASH_INIT, ROOT_INIT);
    let msg = InstantiateMsg {
        client_state: encode(&cs),
        consensus_state: encode(&cons),
        checksum: Binary::default(),
    };
    instantiate(deps.as_mut(), env, info, msg).expect("instantiate");
}

#[test]
fn instantiate_stores_state_and_consensus() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    let latest: LatestHeightResult = serde_json::from_slice(
        query(deps.as_ref(), mock_env(), QueryMsg::LatestHeight {})
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    assert_eq!(latest.height.revision_height, 100);

    let status: StatusResult = serde_json::from_slice(
        query(deps.as_ref(), mock_env(), QueryMsg::Status {})
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    assert_eq!(status.status, ClientStatus::Active);

    let ts: TimestampAtHeightResult = serde_json::from_slice(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TimestampAtHeight {
                height: MsgHeight {
                    revision_number: 0,
                    revision_height: 100,
                },
            },
        )
        .unwrap()
        .as_slice(),
    )
    .unwrap();
    assert_eq!(ts.timestamp, 1_000_000);
}

#[test]
fn instantiate_rejects_double_instantiation() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    let cs = fresh_client_state(100);
    let cons = fresh_consensus_state(1_000_000, LEDGER_HASH_INIT, ROOT_INIT);
    let msg = InstantiateMsg {
        client_state: encode(&cs),
        consensus_state: encode(&cons),
        checksum: Binary::default(),
    };
    let info = message_info(&deps.api.addr_make("creator"), &[]);
    let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert!(matches!(err, ContractError::AlreadyInitialised));
}

#[test]
fn update_state_advances_height_when_chain_intact() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    let hdr = header(
        100,
        105,
        1_000_500,
        LEDGER_HASH_INIT,
        LEDGER_HASH_NEXT,
        ROOT_NEXT,
    );
    let msg = SudoMsg::UpdateState(UpdateStateMsg {
        client_message: encode(&hdr),
    });
    let resp = sudo(deps.as_mut(), mock_env(), msg).expect("update_state");
    let result: UpdateStateResult = serde_json::from_slice(resp.data.unwrap().as_slice()).unwrap();
    assert_eq!(result.heights.len(), 1);
    assert_eq!(result.heights[0].revision_height, 105);

    let latest: LatestHeightResult = serde_json::from_slice(
        query(deps.as_ref(), mock_env(), QueryMsg::LatestHeight {})
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    assert_eq!(latest.height.revision_height, 105);
}

#[test]
fn update_state_rejects_non_advancing_height() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    let hdr = header(
        100,
        100,
        1_000_500,
        LEDGER_HASH_INIT,
        LEDGER_HASH_NEXT,
        ROOT_NEXT,
    );
    let msg = SudoMsg::UpdateState(UpdateStateMsg {
        client_message: encode(&hdr),
    });
    let err = sudo(deps.as_mut(), mock_env(), msg).unwrap_err();
    assert!(matches!(
        err,
        ContractError::NonAdvancingHeight {
            trusted: 100,
            target: 100
        }
    ));
}

#[test]
fn update_state_rejects_broken_ledger_chain() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    let bogus_previous = [0xff; 32];
    let hdr = header(
        100,
        105,
        1_000_500,
        bogus_previous,
        LEDGER_HASH_NEXT,
        ROOT_NEXT,
    );
    let msg = SudoMsg::UpdateState(UpdateStateMsg {
        client_message: encode(&hdr),
    });
    let err = sudo(deps.as_mut(), mock_env(), msg).unwrap_err();
    assert!(matches!(
        err,
        ContractError::LedgerHashChainBroken { .. }
    ));
}

#[test]
fn check_for_misbehaviour_detects_conflicting_root_at_same_height() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    let hdr1 = header(
        100,
        105,
        1_000_500,
        LEDGER_HASH_INIT,
        LEDGER_HASH_NEXT,
        ROOT_NEXT,
    );
    sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::UpdateState(UpdateStateMsg {
            client_message: encode(&hdr1),
        }),
    )
    .unwrap();

    let conflicting_root = [0xCC; 32];
    let hdr2 = header(
        100,
        105,
        1_000_700,
        LEDGER_HASH_INIT,
        LEDGER_HASH_NEXT,
        conflicting_root,
    );
    let resp = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::CheckForMisbehaviour(CheckForMisbehaviourMsg {
            client_message: encode(&hdr2),
        }),
    )
    .unwrap();
    let result: CheckForMisbehaviourResult =
        serde_json::from_slice(resp.data.unwrap().as_slice()).unwrap();
    assert!(result.found_misbehaviour);
}

#[test]
fn update_state_on_misbehaviour_freezes_client() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::UpdateStateOnMisbehaviour(UpdateStateOnMisbehaviourMsg {
            client_message: Binary::default(),
        }),
    )
    .unwrap();

    let status: StatusResult = serde_json::from_slice(
        query(deps.as_ref(), mock_env(), QueryMsg::Status {})
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    assert_eq!(status.status, ClientStatus::Frozen);
}

#[test]
fn update_state_rejects_when_frozen() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::UpdateStateOnMisbehaviour(UpdateStateOnMisbehaviourMsg {
            client_message: Binary::default(),
        }),
    )
    .unwrap();

    let hdr = header(
        100,
        105,
        1_000_500,
        LEDGER_HASH_INIT,
        LEDGER_HASH_NEXT,
        ROOT_NEXT,
    );
    let err = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::UpdateState(UpdateStateMsg {
            client_message: encode(&hdr),
        }),
    )
    .unwrap_err();
    assert!(matches!(err, ContractError::Frozen { .. }));
}

#[test]
fn verify_membership_rejects_when_consensus_state_missing() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    let err = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::VerifyMembership(VerifyMembershipMsg {
            height: MsgHeight {
                revision_number: 0,
                revision_height: 999,
            },
            delay_time_period: 0,
            delay_block_period: 0,
            proof: Binary::default(),
            path: vec![],
            value: Binary::default(),
        }),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ContractError::ConsensusStateMissing { height: 999 }
    ));
}

#[test]
fn verify_membership_accepts_when_consensus_state_exists() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::VerifyMembership(VerifyMembershipMsg {
            height: MsgHeight {
                revision_number: 0,
                revision_height: 100,
            },
            delay_time_period: 0,
            delay_block_period: 0,
            proof: Binary::default(),
            path: vec![],
            value: Binary::default(),
        }),
    )
    .expect("verify_membership stub passes for known height");
}

#[test]
fn verify_non_membership_rejects_when_frozen() {
    let mut deps = mock_dependencies();
    do_instantiate(&mut deps);

    sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::UpdateStateOnMisbehaviour(UpdateStateOnMisbehaviourMsg {
            client_message: Binary::default(),
        }),
    )
    .unwrap();

    let err = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::VerifyNonMembership(VerifyNonMembershipMsg {
            height: MsgHeight {
                revision_number: 0,
                revision_height: 100,
            },
            delay_time_period: 0,
            delay_block_period: 0,
            proof: Binary::default(),
            path: vec![],
        }),
    )
    .unwrap_err();
    assert!(matches!(err, ContractError::Frozen { .. }));
}
