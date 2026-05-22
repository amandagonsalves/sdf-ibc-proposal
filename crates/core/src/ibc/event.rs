use ibc::core::handler::types::events::IbcEvent as RawIbcEvent;

#[derive(Debug, Clone)]
pub enum IbcEvent {
    ClientCreated,
    ClientUpdated,
    PacketReceived,
    PacketAcknowledged,
    PacketTimeout,
    Other,
}

impl From<RawIbcEvent> for IbcEvent {
    fn from(event: RawIbcEvent) -> Self {
        match event {
            RawIbcEvent::CreateClient(_) => Self::ClientCreated,
            RawIbcEvent::UpdateClient(_) => Self::ClientUpdated,
            RawIbcEvent::ReceivePacket(_) => Self::PacketReceived,
            RawIbcEvent::AcknowledgePacket(_) => Self::PacketAcknowledged,
            RawIbcEvent::TimeoutPacket(_) => Self::PacketTimeout,
            _ => Self::Other,
        }
    }
}
