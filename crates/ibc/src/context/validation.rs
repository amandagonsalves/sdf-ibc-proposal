use core::time::Duration;

use ibc::core::{
    channel::types::{
        channel::ChannelEnd,
        commitment::{AcknowledgementCommitment, PacketCommitment},
        packet::Receipt,
    },
    client::context::client_state::ClientStateValidation,
    client::types::Height,
    commitment_types::commitment::CommitmentPrefix,
    connection::types::ConnectionEnd,
    host::types::{
        error::HostError,
        identifiers::{ConnectionId, Sequence},
        path::{
            AckPath, ChannelEndPath, ClientConnectionPath, CommitmentPath, ConnectionPath,
            ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
        },
    },
    primitives::{Signer, Timestamp},
    router::ValidationContext,
};

use crate::context::{
    client::{MockClientState, MockConsensusState},
    StellarIbcContext,
    storage::SorobanStorage,
};

impl<S: SorobanStorage> ValidationContext for StellarIbcContext<S> {
    type V = Self;
    type HostClientState = MockClientState;
    type HostConsensusState = MockConsensusState;

    fn get_client_validation_context(&self) -> &Self::V {
        self
    }

    fn host_height(&self) -> Result<Height, HostError> {
        Err(HostError::missing_state("host_height: not implemented"))
    }

    fn host_timestamp(&self) -> Result<Timestamp, HostError> {
        Err(HostError::missing_state("host_timestamp: not implemented"))
    }

    fn host_consensus_state(&self, _height: &Height) -> Result<Self::HostConsensusState, HostError> {
        Err(HostError::missing_state("host_consensus_state: not implemented"))
    }

    fn client_counter(&self) -> Result<u64, HostError> {
        Err(HostError::missing_state("client_counter: not implemented"))
    }

    fn connection_end(&self, _conn_id: &ConnectionId) -> Result<ConnectionEnd, HostError> {
        Err(HostError::missing_state("connection_end: not implemented"))
    }

    fn validate_self_client(
        &self,
        _client_state_of_host_on_counterparty: Self::HostClientState,
    ) -> Result<(), HostError> {
        Err(HostError::invalid_state("validate_self_client: not implemented"))
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from(b"ibc".to_vec()).expect("'ibc' is valid prefix")
    }

    fn connection_counter(&self) -> Result<u64, HostError> {
        Err(HostError::missing_state("connection_counter: not implemented"))
    }

    fn channel_end(&self, _channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, HostError> {
        Err(HostError::missing_state("channel_end: not implemented"))
    }

    fn get_next_sequence_send(&self, _seq_send_path: &SeqSendPath) -> Result<Sequence, HostError> {
        Err(HostError::missing_state("get_next_sequence_send: not implemented"))
    }

    fn get_next_sequence_recv(&self, _seq_recv_path: &SeqRecvPath) -> Result<Sequence, HostError> {
        Err(HostError::missing_state("get_next_sequence_recv: not implemented"))
    }

    fn get_next_sequence_ack(&self, _seq_ack_path: &SeqAckPath) -> Result<Sequence, HostError> {
        Err(HostError::missing_state("get_next_sequence_ack: not implemented"))
    }

    fn get_packet_commitment(
        &self,
        _commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, HostError> {
        Err(HostError::missing_state("get_packet_commitment: not implemented"))
    }

    fn get_packet_receipt(&self, _receipt_path: &ReceiptPath) -> Result<Receipt, HostError> {
        Err(HostError::missing_state("get_packet_receipt: not implemented"))
    }

    fn get_packet_acknowledgement(
        &self,
        _ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, HostError> {
        Err(HostError::missing_state("get_packet_acknowledgement: not implemented"))
    }

    fn channel_counter(&self) -> Result<u64, HostError> {
        Err(HostError::missing_state("channel_counter: not implemented"))
    }

    fn max_expected_time_per_block(&self) -> Duration {
        Duration::from_secs(6)
    }

    fn validate_message_signer(&self, _signer: &Signer) -> Result<(), HostError> {
        Ok(())
    }
}

const _: () = {
    fn _assert_client_state_validation<V: ibc::core::client::context::ClientValidationContext, T: ClientStateValidation<V>>() {}
    fn _check() {
    }
};