extern crate alloc;

use alloc::vec::Vec;

use ibc_core::client::types::Height;
use ibc_core::host::types::error::DecodingError;
use ibc_core::primitives::proto::Any;

pub const TYPE_URL: &str = "/ibc.lightclients.stellar.v1.Header";

#[derive(Clone, Debug, PartialEq)]
pub struct ScpEnvelope {
    pub node_id: [u8; 32],
    pub statement_xdr: Vec<u8>,
    pub signature: [u8; 64],
}

#[derive(Clone, Debug, PartialEq)]
pub struct StellarHeader {
    pub ledger_header_xdr: Vec<u8>,
    pub scp_envelopes: Vec<ScpEnvelope>,
    pub trusted_height: Height,
    pub trusted_validators: Vec<[u8; 32]>,
}

impl TryFrom<Any> for StellarHeader {
    type Error = DecodingError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        if raw.type_url != TYPE_URL {
            return Err(DecodingError::MismatchedResourceName {
                expected: TYPE_URL.into(),
                actual: raw.type_url,
            });
        }
        Err(DecodingError::MissingRawData {
            description: "proto decode not yet wired".into(),
        })
    }
}

impl From<StellarHeader> for Any {
    fn from(h: StellarHeader) -> Self {
        let _ = h;
        Any {
            type_url: TYPE_URL.into(),
            value: alloc::vec![],
        }
    }
}
