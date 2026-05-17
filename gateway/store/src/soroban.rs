use soroban_client::Server;

use crate::key::SorobanStorageKey;

pub struct SorobanClient {
    pub(crate) server: Server,
}

impl SorobanClient {
    pub fn new(rpc_url: &str) -> anyhow::Result<Self> {
        let server = Server::new(rpc_url, soroban_client::Options::default())
            .map_err(|e| anyhow::anyhow!("failed to connect to Soroban RPC: {e}"))?;
        Ok(Self { server })
    }

    pub async fn get_ledger_entry(
        &self,
        _contract_id: &str,
        _key: &SorobanStorageKey,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        todo!("call getLedgerEntries on Soroban RPC and decode ScVal")
    }

    pub async fn submit_and_wait(
        &self,
        _tx_xdr: &str,
        _network_passphrase: &str,
    ) -> anyhow::Result<Vec<Vec<u8>>> {
        todo!("submit transaction via Soroban RPC and collect ContractEvents")
    }

    pub async fn latest_ledger_sequence(&self) -> anyhow::Result<u32> {
        todo!("call getLatestLedger on Soroban RPC")
    }
}
