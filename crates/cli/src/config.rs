use std::env;
use std::path::Path;

pub struct Config {
    pub cosmos_chain_id: String,
    pub cosmos_rest_url: String,
    pub cosmos_rpc_url: String,
    pub api_url: String,
    pub gateway_grpc_addr: String,
    pub hermes_config: String,
    pub stellar_signing_key: String,
    pub ibc_contract_id: String,
    pub transfer_contract_id: String,
    pub deployer_address: String,
    pub stellar_client_id: String,
}

impl Config {
    pub fn load(root: &Path) -> Self {
        let _ = dotenvy::from_path(root.join(".env"));

        let get = |key: &str, default: &str| env::var(key).unwrap_or_else(|_| default.to_string());

        let api_port = get("STELLAR_API_PORT", "8101");
        let grpc_port = get("STELLAR_GATEWAY_GRPC_PORT", "50052");

        Self {
            cosmos_chain_id: get("COSMOS_CHAIN_ID", "localosmosis"),
            cosmos_rest_url: get("COSMOS_REST_URL", "http://127.0.0.1:1318"),
            cosmos_rpc_url: get("COSMOS_RPC_URL", "http://127.0.0.1:26658"),
            api_url: env::var("STELLAR_API_URL")
                .unwrap_or_else(|_| format!("http://127.0.0.1:{api_port}")),
            gateway_grpc_addr: format!("127.0.0.1:{grpc_port}"),
            hermes_config: get(
                "HERMES_CONFIG",
                &root.join("ci/hermes-config.toml").display().to_string(),
            ),
            stellar_signing_key: get("STELLAR_SIGNING_KEY", ""),
            ibc_contract_id: get("IBC_CONTRACT_ID", ""),
            transfer_contract_id: get("TRANSFER_CONTRACT_ID", ""),
            deployer_address: get("DEPLOYER_ADDRESS", ""),
            stellar_client_id: get("STELLAR_CLIENT_ID", ""),
        }
    }

    pub fn cosmos_node_info_url(&self) -> String {
        format!(
            "{}/cosmos/base/tendermint/v1beta1/node_info",
            self.cosmos_rest_url
        )
    }

    pub fn api_health_url(&self) -> String {
        format!("{}/health", self.api_url)
    }

    pub fn clients_url(&self) -> String {
        format!("{}/stellar/clients", self.api_url)
    }
}
