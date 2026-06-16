use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{Path, State as AxState};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use soroban_client::xdr::{
    ContractDataDurability, ContractDataEntry, ContractId, ExtensionPoint, Hash, LedgerCloseMeta,
    LedgerCloseMetaV0, LedgerEntry, LedgerEntryChange, LedgerEntryChanges, LedgerEntryData,
    LedgerEntryExt, Limits, OperationMeta, ScVal, TransactionMeta, TransactionMetaV1,
    TransactionResultMeta, VecM, WriteXdr,
};
use tokio::net::TcpListener;
use tonic::transport::Channel;

use stellar_hermes_gateway::config::GatewayConfig;
use stellar_hermes_gateway::proto::stellar_gateway_msg_client::StellarGatewayMsgClient;
use stellar_hermes_gateway::proto::stellar_gateway_query_client::StellarGatewayQueryClient;
use stellar_hermes_gateway::runner;
use stellar_ibc_core::conversion::{scval_bytes, scval_struct};

pub struct PrepareCall {
    pub signer: String,
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Default)]
pub struct MockData {
    pub latest_ledger: u32,
    pub prepare_tx_xdr: Vec<u8>,
    pub submit_hash: String,
    pub submit_return_value_xdr: Vec<u8>,
    pub ledgers: HashMap<u32, (Vec<u8>, Option<Vec<u8>>)>,
    pub client_states: HashMap<String, Vec<u8>>,
    pub consensus_states: HashMap<String, Vec<u8>>,
    pub events: Vec<Value>,
    pub prepare_calls: Vec<PrepareCall>,
    pub submit_calls: Vec<String>,
}

pub struct GatewayTest {
    data: Arc<Mutex<MockData>>,
    msg: StellarGatewayMsgClient<Channel>,
    query: StellarGatewayQueryClient<Channel>,
}

impl GatewayTest {
    pub async fn start(ibc_contract_id: Option<[u8; 32]>) -> Self {
        let (api_url, data) = start_mock_api().await;

        let id_string = ibc_contract_id
            .map(|c| format!("{}", stellar_strkey::Contract(c)))
            .unwrap_or_default();
        let cfg = GatewayConfig {
            host: "127.0.0.1".to_string(),
            grpc_port: 0,
            api_url,
            ibc_contract_id: id_string,
        };

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            runner::serve_with_listener(cfg, listener).await.unwrap();
        });

        let channel = connect(format!("http://{addr}")).await;

        Self {
            data,
            msg: StellarGatewayMsgClient::new(channel.clone()),
            query: StellarGatewayQueryClient::new(channel),
        }
    }

    pub fn msg(&self) -> StellarGatewayMsgClient<Channel> {
        self.msg.clone()
    }

    pub fn query(&self) -> StellarGatewayQueryClient<Channel> {
        self.query.clone()
    }

    pub fn with_data<R>(&self, f: impl FnOnce(&mut MockData) -> R) -> R {
        let mut d = self.data.lock().unwrap();
        f(&mut d)
    }
}

async fn connect(endpoint: String) -> Channel {
    for _ in 0..100 {
        if let Ok(channel) = Channel::from_shared(endpoint.clone())
            .unwrap()
            .connect()
            .await
        {
            return channel;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("could not connect to gateway at {endpoint}");
}

async fn start_mock_api() -> (String, Arc<Mutex<MockData>>) {
    let data = Arc::new(Mutex::new(MockData::default()));
    let app = Router::new()
        .route("/ledger/latest", get(ledger_latest))
        .route("/ledger/{seq}", get(ledger_by_seq))
        .route("/tx/prepare", post(tx_prepare))
        .route("/tx/submit", post(tx_submit))
        .route("/events", get(events))
        .route("/stellar/clients", get(list_clients))
        .route("/stellar/clients/{id}/state", get(client_state))
        .route("/stellar/clients/{id}/consensus/{height}", get(consensus_state))
        .route(
            "/stellar/transfer/balance/{denom}/{address}",
            get(transfer_balance),
        )
        .with_state(data.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{addr}"), data)
}

async fn ledger_latest(AxState(data): AxState<Arc<Mutex<MockData>>>) -> Json<Value> {
    let d = data.lock().unwrap();
    Json(json!({ "sequence": d.latest_ledger }))
}

async fn ledger_by_seq(
    AxState(data): AxState<Arc<Mutex<MockData>>>,
    Path(seq): Path<u32>,
) -> Json<Value> {
    let d = data.lock().unwrap();
    let (header, meta) = d
        .ledgers
        .get(&seq)
        .cloned()
        .unwrap_or_else(|| (vec![], None));
    let mut out = json!({ "header_xdr": hex::encode(header) });
    if let Some(meta) = meta {
        out["metadata_xdr"] = json!(hex::encode(meta));
    }
    Json(out)
}

async fn tx_prepare(
    AxState(data): AxState<Arc<Mutex<MockData>>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let signer = body
        .get("signer")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let method = body
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let args = body
        .get("args_xdr")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .filter_map(|s| hex::decode(s).ok())
                .collect()
        })
        .unwrap_or_default();

    let mut d = data.lock().unwrap();
    d.prepare_calls.push(PrepareCall {
        signer,
        method,
        args,
    });
    let tx_xdr = hex::encode(&d.prepare_tx_xdr);
    Json(json!({ "tx_xdr": tx_xdr }))
}

async fn tx_submit(
    AxState(data): AxState<Arc<Mutex<MockData>>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let tx = body
        .get("tx_xdr")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let mut d = data.lock().unwrap();
    d.submit_calls.push(tx);
    let mut out = json!({ "hash": d.submit_hash });
    if !d.submit_return_value_xdr.is_empty() {
        out["return_value_xdr"] = json!(hex::encode(&d.submit_return_value_xdr));
    }
    Json(out)
}

async fn events(AxState(data): AxState<Arc<Mutex<MockData>>>) -> Json<Value> {
    let d = data.lock().unwrap();
    Json(json!({
        "latest_ledger": d.latest_ledger,
        "cursor": "",
        "events": d.events.clone(),
    }))
}

async fn list_clients(AxState(data): AxState<Arc<Mutex<MockData>>>) -> Json<Value> {
    let d = data.lock().unwrap();
    let ids: Vec<String> = d.client_states.keys().cloned().collect();
    Json(json!({ "clients": [ { "client_ids": ids } ] }))
}

async fn client_state(
    AxState(data): AxState<Arc<Mutex<MockData>>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let d = data.lock().unwrap();
    match d.client_states.get(&id) {
        Some(xdr) => Json(json!({ "client_state_xdr": hex::encode(xdr) })),
        _ => Json(json!({})),
    }
}

async fn consensus_state(
    AxState(data): AxState<Arc<Mutex<MockData>>>,
    Path((id, _height)): Path<(String, u64)>,
) -> Json<Value> {
    let d = data.lock().unwrap();
    match d.consensus_states.get(&id) {
        Some(xdr) => Json(json!({ "consensus_state_xdr": hex::encode(xdr) })),
        _ => Json(json!({})),
    }
}

async fn transfer_balance(
    AxState(_data): AxState<Arc<Mutex<MockData>>>,
    Path((_denom, _address)): Path<(String, String)>,
) -> Json<Value> {
    Json(json!({ "balance": "0" }))
}

pub fn sample_packet() -> ScVal {
    scval_struct(vec![
        ("sequence", ScVal::U64(1)),
        ("data", scval_bytes(b"payload").unwrap()),
    ])
    .unwrap()
}

pub fn ledger_meta_with_write(contract: [u8; 32], key: Vec<u8>, val: Vec<u8>) -> Vec<u8> {
    let entry = ContractDataEntry {
        ext: ExtensionPoint::V0,
        contract: soroban_client::xdr::ScAddress::Contract(ContractId(Hash(contract))),
        key: scval_bytes(&key).unwrap(),
        durability: ContractDataDurability::Persistent,
        val: scval_bytes(&val).unwrap(),
    };
    let ledger_entry = LedgerEntry {
        last_modified_ledger_seq: 0,
        data: LedgerEntryData::ContractData(entry),
        ext: LedgerEntryExt::V0,
    };
    let changes =
        LedgerEntryChanges(vec![LedgerEntryChange::Created(ledger_entry)].try_into().unwrap());
    let tx_meta = TransactionMeta::V1(TransactionMetaV1 {
        tx_changes: LedgerEntryChanges(VecM::default()),
        operations: vec![OperationMeta { changes }].try_into().unwrap(),
    });
    let tx_result = TransactionResultMeta {
        result: Default::default(),
        fee_processing: LedgerEntryChanges(VecM::default()),
        tx_apply_processing: tx_meta,
    };
    let v0 = LedgerCloseMetaV0 {
        ledger_header: Default::default(),
        tx_set: Default::default(),
        tx_processing: vec![tx_result].try_into().unwrap(),
        upgrades_processing: VecM::default(),
        scp_info: VecM::default(),
    };
    LedgerCloseMeta::V0(v0).to_xdr(Limits::none()).unwrap()
}
