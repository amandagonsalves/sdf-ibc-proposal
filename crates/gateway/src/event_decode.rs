use soroban_client::xdr::{Limits, ReadXdr, ScMap, ScVal};

fn as_symbol(v: &ScVal) -> Option<String> {
    match v {
        ScVal::Symbol(s) => Some(s.to_string()),
        _ => None,
    }
}

fn as_string(v: &ScVal) -> Option<String> {
    match v {
        ScVal::String(s) => Some(s.to_string()),
        _ => None,
    }
}

fn as_u64(v: &ScVal) -> Option<u64> {
    match v {
        ScVal::U64(n) => Some(*n),
        _ => None,
    }
}

fn as_map(v: &ScVal) -> Option<&ScMap> {
    match v {
        ScVal::Map(Some(m)) => Some(m),
        _ => None,
    }
}

fn field<'a>(m: &'a ScMap, key: &str) -> Option<&'a ScVal> {
    m.0.iter()
        .find(|e| matches!(&e.key, ScVal::Symbol(s) if s.to_string() == key))
        .map(|e| &e.val)
}

fn first_payload_ports(packet: &ScMap) -> (String, String) {
    let default = || ("transfer".to_string(), "transfer".to_string());

    let Some(ScVal::Vec(Some(payloads))) = field(packet, "payloads") else {
        return default();
    };
    let Some(payload) = payloads.0.first().and_then(as_map) else {
        return default();
    };

    (
        field(payload, "source_port")
            .and_then(as_string)
            .unwrap_or_else(|| "transfer".to_string()),
        field(payload, "dest_port")
            .and_then(as_string)
            .unwrap_or_else(|| "transfer".to_string()),
    )
}

pub fn event_attributes(topics_xdr: &[Vec<u8>], value_xdr: &[u8]) -> Option<String> {
    let kind = ScVal::from_xdr(topics_xdr.first()?, Limits::none())
        .ok()
        .as_ref()
        .and_then(as_symbol)?;

    let value = ScVal::from_xdr(value_xdr, Limits::none()).ok()?;
    let root = as_map(&value)?;

    if let Some(packet) = field(root, "packet").and_then(as_map) {
        let sequence = field(packet, "sequence").and_then(as_u64).unwrap_or(0);
        let source_client = field(packet, "source_client")
            .and_then(as_string)
            .unwrap_or_default();
        let dest_client = field(packet, "dest_client")
            .and_then(as_string)
            .unwrap_or_default();
        let (source_port, dest_port) = first_payload_ports(packet);

        let mut text = format!("type={kind}\npacket_sequence={sequence}\n");
        if !source_client.is_empty() {
            text.push_str(&format!("packet_src_channel={source_client}\n"));
        }
        if !dest_client.is_empty() {
            text.push_str(&format!("packet_dst_channel={dest_client}\n"));
        }
        text.push_str(&format!(
            "packet_src_port={source_port}\npacket_dst_port={dest_port}\n"
        ));

        return Some(text);
    }

    let mut text = format!("type={kind}\n");
    if let Some(client_id) = field(root, "client_id").and_then(as_string) {
        text.push_str(&format!("client_id={client_id}\n"));
    }

    Some(text)
}
