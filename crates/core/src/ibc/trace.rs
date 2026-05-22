use ibc::core::host::types::identifiers::{ChannelId, PortId};

pub fn ibc_token(trace: &str) -> String {
    trace.to_string()
}

pub fn is_receiver_chain_source(trace: &str, src_port: &PortId, src_channel: &ChannelId) -> bool {
    let prefix = format!("{src_port}/{src_channel}/");
    trace.starts_with(&prefix)
}
