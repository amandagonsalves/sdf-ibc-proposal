use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use soroban_client::xdr::{
    ContractDataDurability, ContractId, Hash, LedgerEntryData, LedgerKey, LedgerKeyContractData,
    Limits, ReadXdr, ScAddress, ScString, ScSymbol, ScVal, ScVec, StringM, VecM, WriteXdr,
};

use crate::state::AppState;

#[derive(Serialize)]
pub struct BalanceResponse {
    balance: String,
}

fn err<E: std::fmt::Display>(status: StatusCode, e: E) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": e.to_string() })))
}

fn balance_ledger_key(contract: [u8; 32], address_hex: &str, denom: &str) -> anyhow::Result<Vec<u8>> {
    let variant: StringM<32> = "Balance".try_into()?;
    let addr_xdr = hex::decode(address_hex)?;
    let addr_val = ScVal::from_xdr(&addr_xdr, Limits::none())
        .map_err(|e| anyhow::anyhow!("address ScVal decode: {e}"))?;
    let denom_str: StringM = denom.try_into()?;
    let key_val = ScVal::Vec(Some(ScVec(VecM::try_from(vec![
        ScVal::Symbol(ScSymbol(variant)),
        addr_val,
        ScVal::String(ScString(denom_str)),
    ])?)));
    let key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract(ContractId(Hash(contract))),
        key: key_val,
        durability: ContractDataDurability::Persistent,
    });
    Ok(key.to_xdr(Limits::none())?)
}

fn decode_i128(entry_xdr: &[u8]) -> Option<i128> {
    match LedgerEntryData::from_xdr(entry_xdr, Limits::none()).ok()? {
        LedgerEntryData::ContractData(d) => match d.val {
            ScVal::I128(parts) => Some(((parts.hi as i128) << 64) | (parts.lo as i128)),
            _ => None,
        },
        _ => None,
    }
}

#[tracing::instrument(skip(_state))]
pub async fn balance(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    tracing::debug!(%address, "GET /balance/{address}");
    (
        StatusCode::OK,
        Json(BalanceResponse {
            balance: "0".to_string(),
        }),
    )
}

#[tracing::instrument(skip(state))]
pub async fn transfer_balance(
    State(state): State<Arc<AppState>>,
    Path((denom, address)): Path<(String, String)>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<Value>)> {
    tracing::debug!(%denom, %address, "GET /stellar/transfer/balance");

    if state.transfer_contract_id.is_empty() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            "TRANSFER_CONTRACT_ADDRESS not configured",
        ));
    }

    let contract = stellar_strkey::Contract::from_string(state.transfer_contract_id.as_str())
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("transfer contract addr: {e}")))?
        .0;

    let key = balance_ledger_key(contract, &address, &denom)
        .map_err(|e| err(StatusCode::BAD_REQUEST, e))?;

    let entry = state
        .rpc
        .get_ledger_entry(&key)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, e))?;

    let balance = entry.and_then(|x| decode_i128(&x)).unwrap_or(0);

    Ok(Json(BalanceResponse {
        balance: balance.to_string(),
    }))
}
