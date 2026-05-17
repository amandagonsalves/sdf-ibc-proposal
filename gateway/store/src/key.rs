#[derive(Debug, Clone)]
pub enum SorobanStorageKey {
    ClientState(String),

    ConsensusState {
        client_id: String,
        revision: u64,
        height: u64,
    },

    Counterparty(String),

    Commitment {
        client_id: String,
        seq: u64,
    },

    Receipt {
        client_id: String,
        seq: u64,
    },

    Ack {
        client_id: String,
        seq: u64,
    },

    NextSeqSend(String),
}

impl SorobanStorageKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!("encode ICS-24 path bytes — coordinate with soroban-ibc contract storage.rs")
    }
}
