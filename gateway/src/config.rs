use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub host: String,
    pub grpc_port: u16,
    pub http_port: u16,
    pub rpc_url: String,
    pub ibc_contract_id: String,
    pub transfer_contract_id: String,
    pub network_passphrase: String,
    pub signing_key: String,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("STELLAR_GATEWAY_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            grpc_port: std::env::var("STELLAR_GATEWAY_GRPC_PORT")
                .unwrap_or_else(|_| "50052".to_string())
                .parse()
                .expect("STELLAR_GATEWAY_GRPC_PORT must be a valid port number"),
            http_port: std::env::var("STELLAR_GATEWAY_HTTP_PORT")
                .expect("STELLAR_GATEWAY_HTTP_PORT must be set")
                .parse()
                .expect("STELLAR_GATEWAY_HTTP_PORT must be a valid port number"),
            rpc_url: std::env::var("STELLAR_RPC_URL").expect("STELLAR_RPC_URL must be set"),
            ibc_contract_id: std::env::var("STELLAR_IBC_CONTRACT_ID").unwrap_or_default(),
            transfer_contract_id: std::env::var("STELLAR_TRANSFER_CONTRACT_ID").unwrap_or_default(),
            network_passphrase: std::env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
            signing_key: std::env::var("STELLAR_SIGNING_KEY")
                .expect("STELLAR_SIGNING_KEY must be set"),
        }
    }

    pub fn grpc_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.grpc_port)
            .parse()
            .expect("invalid grpc address")
    }

    pub fn http_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.http_port)
            .parse()
            .expect("invalid http address")
    }
}
