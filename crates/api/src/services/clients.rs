use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use soroban_client::xdr::{
    ContractDataDurability, ContractId, Hash, LedgerEntryData, LedgerKey, LedgerKeyContractData,
    Limits, ReadXdr, ScAddress, ScString, ScSymbol, ScVal, ScVec, StringM, VecM, WriteXdr,
};

use crate::AppState;

const DEFAULT_CLIENT_TYPES: &[&str] = &["07-tendermint", "mock", "attestation", "08-wasm"];

#[derive(Deserialize)]
pub struct ListClientsQuery {
    pub client_type: Option<String>,
}

fn err<E: std::fmt::Display>(status: StatusCode, e: E) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": e.to_string() })))
}

fn next_client_id_ledger_key(router: [u8; 32], client_type: &str) -> anyhow::Result<Vec<u8>> {
    let variant: StringM<32> = "NextClientId".try_into()?;
    let type_str: StringM = client_type.try_into()?;
    let key_val = ScVal::Vec(Some(ScVec(VecM::try_from(vec![
        ScVal::Symbol(ScSymbol(variant)),
        ScVal::String(ScString(type_str)),
    ])?)));
    let key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract(ContractId(Hash(router))),
        key: key_val,
        durability: ContractDataDurability::Persistent,
    });
    Ok(key.to_xdr(Limits::none())?)
}

fn decode_counter(entry_xdr: &[u8]) -> Option<u32> {
    match LedgerEntryData::from_xdr(entry_xdr, Limits::none()).ok()? {
        LedgerEntryData::ContractData(d) => match d.val {
            ScVal::U32(n) => Some(n),
            _ => None,
        },
        _ => None,
    }
}

fn contract_data_key(contract: [u8; 32], variant: &str, arg: &str) -> anyhow::Result<Vec<u8>> {
    let variant_sym: StringM<32> = variant.try_into()?;
    let arg_str: StringM = arg.try_into()?;
    let key_val = ScVal::Vec(Some(ScVec(VecM::try_from(vec![
        ScVal::Symbol(ScSymbol(variant_sym)),
        ScVal::String(ScString(arg_str)),
    ])?)));
    let key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract(ContractId(Hash(contract))),
        key: key_val,
        durability: ContractDataDurability::Persistent,
    });
    Ok(key.to_xdr(Limits::none())?)
}

fn consensus_data_key(contract: [u8; 32], client_id: &str, height: u64) -> anyhow::Result<Vec<u8>> {
    let variant_sym: StringM<32> = "Consensus".try_into()?;
    let client_str: StringM = client_id.try_into()?;
    let key_val = ScVal::Vec(Some(ScVec(VecM::try_from(vec![
        ScVal::Symbol(ScSymbol(variant_sym)),
        ScVal::String(ScString(client_str)),
        ScVal::U64(height),
    ])?)));
    let key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract(ContractId(Hash(contract))),
        key: key_val,
        durability: ContractDataDurability::Persistent,
    });
    Ok(key.to_xdr(Limits::none())?)
}

fn decode_contract_val(entry_xdr: &[u8]) -> Option<ScVal> {
    match LedgerEntryData::from_xdr(entry_xdr, Limits::none()).ok()? {
        LedgerEntryData::ContractData(d) => Some(d.val),
        _ => None,
    }
}

fn client_type_of(client_id: &str) -> &str {
    match client_id.rfind('-') {
        Some(i) => &client_id[..i],
        None => client_id,
    }
}

#[tracing::instrument(skip(state))]
pub async fn client_state(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("GET /stellar/clients/{client_id}/state");

    if state.ibc_contract_id.is_empty() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            "ROUTER_CONTRACT_ADDRESS not configured",
        ));
    }

    let router = stellar_strkey::Contract::from_string(&state.ibc_contract_id)
        .map_err(|e| {
            err(
                StatusCode::BAD_GATEWAY,
                format!("invalid ROUTER_CONTRACT_ADDRESS: {e}"),
            )
        })?
        .0;

    let lc_key = contract_data_key(router, "ClientLcAddr", &client_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let lc_entry = state
        .rpc
        .get_ledger_entry(&lc_key)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("get lc address: {e}")))?
        .ok_or_else(|| {
            err(
                StatusCode::NOT_FOUND,
                format!("client {client_id} not found"),
            )
        })?;

    let lc_contract = match decode_contract_val(&lc_entry) {
        Some(ScVal::Address(ScAddress::Contract(ContractId(Hash(id))))) => id,
        _ => {
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ClientLcAddr entry is not a contract address",
            ))
        }
    };

    let cs_key = contract_data_key(lc_contract, "Client", &client_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let cs_entry = state
        .rpc
        .get_ledger_entry(&cs_key)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("get client state: {e}")))?
        .ok_or_else(|| {
            err(
                StatusCode::NOT_FOUND,
                format!("client state for {client_id} not found"),
            )
        })?;

    let cs_val = decode_contract_val(&cs_entry).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "client state entry is not contract data",
        )
    })?;
    let cs_xdr = cs_val
        .to_xdr(Limits::none())
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("re-encode: {e}")))?;

    let hex: String = cs_xdr.iter().map(|b| format!("{b:02x}")).collect();

    Ok(Json(json!({
        "client_id": client_id,
        "client_type": client_type_of(&client_id),
        "client_state_xdr": hex,
    })))
}

#[tracing::instrument(skip(state))]
pub async fn consensus_state(
    State(state): State<Arc<AppState>>,
    Path((client_id, height)): Path<(String, u64)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("GET /stellar/clients/{client_id}/consensus/{height}");

    if state.ibc_contract_id.is_empty() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            "ROUTER_CONTRACT_ADDRESS not configured",
        ));
    }

    let router = stellar_strkey::Contract::from_string(&state.ibc_contract_id)
        .map_err(|e| {
            err(
                StatusCode::BAD_GATEWAY,
                format!("invalid ROUTER_CONTRACT_ADDRESS: {e}"),
            )
        })?
        .0;

    let lc_key = contract_data_key(router, "ClientLcAddr", &client_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let lc_entry = state
        .rpc
        .get_ledger_entry(&lc_key)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("get lc address: {e}")))?
        .ok_or_else(|| {
            err(
                StatusCode::NOT_FOUND,
                format!("client {client_id} not found"),
            )
        })?;

    let lc_contract = match decode_contract_val(&lc_entry) {
        Some(ScVal::Address(ScAddress::Contract(ContractId(Hash(id))))) => id,
        _ => {
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ClientLcAddr entry is not a contract address",
            ))
        }
    };

    let cons_key = consensus_data_key(lc_contract, &client_id, height)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let cons_entry = state
        .rpc
        .get_ledger_entry(&cons_key)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("get consensus state: {e}")))?
        .ok_or_else(|| {
            err(
                StatusCode::NOT_FOUND,
                format!("consensus state for {client_id} at height {height} not found"),
            )
        })?;

    let cons_val = decode_contract_val(&cons_entry).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "consensus state entry is not contract data",
        )
    })?;
    let cons_xdr = cons_val
        .to_xdr(Limits::none())
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("re-encode: {e}")))?;

    let hex: String = cons_xdr.iter().map(|b| format!("{b:02x}")).collect();

    Ok(Json(json!({
        "client_id": client_id,
        "client_type": client_type_of(&client_id),
        "height": height,
        "consensus_state_xdr": hex,
    })))
}

#[tracing::instrument(skip(state, params))]
pub async fn list_clients(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListClientsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("GET /stellar/clients");

    if state.ibc_contract_id.is_empty() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            "ROUTER_CONTRACT_ADDRESS not configured",
        ));
    }
    let router = stellar_strkey::Contract::from_string(&state.ibc_contract_id)
        .map_err(|e| {
            err(
                StatusCode::BAD_GATEWAY,
                format!("invalid ROUTER_CONTRACT_ADDRESS: {e}"),
            )
        })?
        .0;

    let types: Vec<String> = match &params.client_type {
        Some(t) => vec![t.clone()],
        None => DEFAULT_CLIENT_TYPES.iter().map(|s| s.to_string()).collect(),
    };

    let mut clients = Vec::new();
    for client_type in &types {
        let key = next_client_id_ledger_key(router, client_type)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        let entry = state.rpc.get_ledger_entry(&key).await.map_err(|e| {
            err(
                StatusCode::BAD_GATEWAY,
                format!("get_ledger_entry({client_type}): {e}"),
            )
        })?;
        let count = entry.as_deref().and_then(decode_counter).unwrap_or(0);
        if count == 0 {
            continue;
        }
        let client_ids: Vec<String> = (0..count).map(|n| format!("{client_type}-{n}")).collect();
        clients.push(json!({
            "client_type": client_type,
            "count": count,
            "client_ids": client_ids,
        }));
    }

    Ok(Json(json!({ "clients": clients })))
}
