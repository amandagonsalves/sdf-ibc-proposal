use ibc::core::{
    channel::types::{
        channel::ChannelEnd,
        commitment::{AcknowledgementCommitment, PacketCommitment},
        packet::Receipt,
    },
    client::context::ClientExecutionContext,
    client::types::Height,
    connection::types::ConnectionEnd,
    handler::types::events::IbcEvent as RawIbcEvent,
    host::types::{
        error::HostError,
        identifiers::{ConnectionId, Sequence},
        path::{
            AckPath, ChannelEndPath, ClientConnectionPath, CommitmentPath, ConnectionPath,
            ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
        },
    },
    router::ExecutionContext,
};

use crate::{
    context::{StellarIbcContext, storage::SorobanStorage},
    event::IbcEvent,
};

impl<S: SorobanStorage> ExecutionContext for StellarIbcContext<S> {
    type E = Self;

    fn get_client_execution_context(&mut self) -> &mut Self::E {
        self
    }

    fn increase_client_counter(&mut self) -> Result<(), HostError> {
        Err(HostError::failed_to_store("increase_client_counter: not implemented"))
    }

    fn store_connection(
        &mut self,
        _connection_path: &ConnectionPath,
        _connection_end: ConnectionEnd,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_connection: not implemented"))
    }

    fn store_connection_to_client(
        &mut self,
        _client_connection_path: &ClientConnectionPath,
        _conn_id: ConnectionId,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_connection_to_client: not implemented"))
    }

    fn increase_connection_counter(&mut self) -> Result<(), HostError> {
        Err(HostError::failed_to_store("increase_connection_counter: not implemented"))
    }

    fn store_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
        _commitment: PacketCommitment,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_packet_commitment: not implemented"))
    }

    fn delete_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("delete_packet_commitment: not implemented"))
    }

    fn store_packet_receipt(
        &mut self,
        _receipt_path: &ReceiptPath,
        _receipt: Receipt,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_packet_receipt: not implemented"))
    }

    fn store_packet_acknowledgement(
        &mut self,
        _ack_path: &AckPath,
        _ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_packet_acknowledgement: not implemented"))
    }

    fn delete_packet_acknowledgement(&mut self, _ack_path: &AckPath) -> Result<(), HostError> {
        Err(HostError::failed_to_store("delete_packet_acknowledgement: not implemented"))
    }

    fn store_channel(
        &mut self,
        _channel_end_path: &ChannelEndPath,
        _channel_end: ChannelEnd,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_channel: not implemented"))
    }

    fn store_next_sequence_send(
        &mut self,
        _seq_send_path: &SeqSendPath,
        _seq: Sequence,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_next_sequence_send: not implemented"))
    }

    fn store_next_sequence_recv(
        &mut self,
        _seq_recv_path: &SeqRecvPath,
        _seq: Sequence,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_next_sequence_recv: not implemented"))
    }

    fn store_next_sequence_ack(
        &mut self,
        _seq_ack_path: &SeqAckPath,
        _seq: Sequence,
    ) -> Result<(), HostError> {
        Err(HostError::failed_to_store("store_next_sequence_ack: not implemented"))
    }

    fn increase_channel_counter(&mut self) -> Result<(), HostError> {
        Err(HostError::failed_to_store("increase_channel_counter: not implemented"))
    }

    fn emit_ibc_event(&mut self, event: RawIbcEvent) -> Result<(), HostError> {
        self.events.push(IbcEvent::from(event));
        Ok(())
    }

    fn log_message(&mut self, message: String) -> Result<(), HostError> {
        tracing::debug!("{}", message);
        Ok(())
    }
}

const _: fn() = || {
    fn _assert<S: SorobanStorage>() {
        fn _check<T: ClientExecutionContext>() {}
        _check::<StellarIbcContext<S>>();
    }
};