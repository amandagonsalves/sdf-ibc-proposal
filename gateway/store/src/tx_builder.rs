pub struct TxBuilder {
    signing_key: String,
    network_passphrase: String,
}

impl TxBuilder {
    pub fn new(signing_key: String, network_passphrase: String) -> Self {
        Self { signing_key, network_passphrase }
    }

    pub async fn build_invocation(
        &self,
        _contract_id: &str,
        _fn_name: &str,
        _args: Vec<Vec<u8>>,
    ) -> anyhow::Result<String> {
        todo!("assemble Soroban InvokeContractOp, sign with Ed25519 key, return XDR envelope")
    }
}
