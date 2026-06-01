use std::env;

use crate::config::{get, ChainId};

const LOCAL_CHAIN_ID: &str = "localosmosis";
const LOCAL_RPC_URL: &str = "http://127.0.0.1:26658";
const LOCAL_REST_URL: &str = "http://127.0.0.1:1318";
const LOCAL_GRPC_URL: &str = "http://127.0.0.1:9094";
const LOCAL_KEY_NAME: &str = "localosmosis";

const TESTNET_CHAIN_ID: &str = "osmo-test-5";
const TESTNET_RPC_URL: &str = "https://osmosis-testnet-rpc.polkachu.com";
const TESTNET_REST_URL: &str = "https://osmosis-testnet-api.polkachu.com";
const TESTNET_GRPC_URL: &str = "http://osmosis-testnet-grpc.polkachu.com:12590";
const TESTNET_KEY_NAME: &str = "osmo-testnet";

pub const COMPOSE_SERVICE: &str = "osmosis";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OsmosisNetwork {
    Local,
    Testnet,
}

pub struct OsmosisConfig {
    pub network: OsmosisNetwork,
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub rest_url: String,
    pub grpc_url: String,
    pub key_name: String,
    pub relayer_mnemonic: String,
}

impl OsmosisConfig {
    pub fn from_env() -> Self {
        let network = match env::var("OSMOSIS_NETWORK").as_deref() {
            Ok("testnet") => OsmosisNetwork::Testnet,
            _ => OsmosisNetwork::Local,
        };

        let (chain_id, rpc, rest, grpc, key) = match network {
            OsmosisNetwork::Local => (
                LOCAL_CHAIN_ID,
                LOCAL_RPC_URL,
                LOCAL_REST_URL,
                LOCAL_GRPC_URL,
                LOCAL_KEY_NAME,
            ),
            OsmosisNetwork::Testnet => (
                TESTNET_CHAIN_ID,
                TESTNET_RPC_URL,
                TESTNET_REST_URL,
                TESTNET_GRPC_URL,
                TESTNET_KEY_NAME,
            ),
        };

        let relayer_mnemonic = get("OSMOSIS_RELAYER_MNEMONIC", "");

        Self {
            network,
            chain_id: ChainId::Cosmos(get("COSMOS_CHAIN_ID", chain_id)),
            rpc_url: get("COSMOS_RPC_URL", rpc),
            rest_url: get("COSMOS_REST_URL", rest),
            grpc_url: get("OSMOSIS_GRPC_URL", grpc),
            key_name: get("OSMOSIS_KEY_NAME", key),
            relayer_mnemonic,
        }
    }

    pub fn is_local(&self) -> bool {
        self.network == OsmosisNetwork::Local
    }

    pub fn status_url(&self) -> String {
        format!("{}/status", self.rpc_url)
    }

    pub fn node_info_url(&self) -> String {
        format!("{}/cosmos/base/tendermint/v1beta1/node_info", self.rest_url)
    }
}
