use serde::Deserialize;

#[derive(Deserialize)]
pub struct SubmitSignedTxRequest {
    pub tx_xdr: String,
}
