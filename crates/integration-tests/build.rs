fn main() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-env=PROTOS_OUT_DIR={}", out_dir.display());

    prost_build::Config::new()
        .file_descriptor_set_path(out_dir.join("stellar_gateway_descriptor.bin"))
        .compile_protos(
            &["../gateway/proto/stellar_gateway.proto"],
            &["../gateway/proto/"],
        )
        .unwrap();

    let codec = "tonic_prost::ProstCodec";

    let query_methods = vec![
        (
            "latest_height",
            "LatestHeight",
            "crate::pb::LatestHeightRequest",
            "crate::pb::LatestHeightResponse",
        ),
        (
            "query_client_state",
            "QueryClientState",
            "crate::pb::QueryClientStateRequest",
            "crate::pb::QueryClientStateResponse",
        ),
        (
            "query_consensus_state",
            "QueryConsensusState",
            "crate::pb::QueryConsensusStateRequest",
            "crate::pb::QueryConsensusStateResponse",
        ),
        (
            "query_packet_commitment",
            "QueryPacketCommitment",
            "crate::pb::QueryPacketCommitmentRequest",
            "crate::pb::QueryPacketCommitmentResponse",
        ),
        (
            "query_packet_receipt",
            "QueryPacketReceipt",
            "crate::pb::QueryPacketReceiptRequest",
            "crate::pb::QueryPacketReceiptResponse",
        ),
        (
            "query_acknowledgement",
            "QueryAcknowledgement",
            "crate::pb::QueryAcknowledgementRequest",
            "crate::pb::QueryAcknowledgementResponse",
        ),
        (
            "query_next_seq_recv",
            "QueryNextSeqRecv",
            "crate::pb::QueryNextSeqRecvRequest",
            "crate::pb::QueryNextSeqRecvResponse",
        ),
        (
            "query_ibc_header",
            "QueryIbcHeader",
            "crate::pb::QueryIbcHeaderRequest",
            "crate::pb::QueryIbcHeaderResponse",
        ),
    ];

    let msg_methods = vec![
        (
            "submit_signed_tx",
            "SubmitSignedTx",
            "crate::pb::SubmitSignedTxRequest",
            "crate::pb::SubmitSignedTxResponse",
        ),
        (
            "create_client",
            "CreateClient",
            "crate::pb::MsgCreateClientRequest",
            "crate::pb::MsgCreateClientResponse",
        ),
        (
            "update_client",
            "UpdateClient",
            "crate::pb::MsgUpdateClientRequest",
            "crate::pb::MsgUpdateClientResponse",
        ),
        (
            "register_counterparty",
            "RegisterCounterparty",
            "crate::pb::MsgRegisterCounterpartyRequest",
            "crate::pb::MsgRegisterCounterpartyResponse",
        ),
        (
            "recv_packet",
            "RecvPacket",
            "crate::pb::MsgRecvPacketRequest",
            "crate::pb::MsgRecvPacketResponse",
        ),
        (
            "ack_packet",
            "AckPacket",
            "crate::pb::MsgAckPacketRequest",
            "crate::pb::MsgAckPacketResponse",
        ),
        (
            "timeout_packet",
            "TimeoutPacket",
            "crate::pb::MsgTimeoutPacketRequest",
            "crate::pb::MsgTimeoutPacketResponse",
        ),
    ];

    let mut query_svc = tonic_build::manual::Service::builder()
        .name("StellarGatewayQuery")
        .package("stellar.gateway.v1");
    for (name, route, input, output) in &query_methods {
        query_svc = query_svc.method(
            tonic_build::manual::Method::builder()
                .name(name)
                .route_name(route)
                .input_type(input)
                .output_type(output)
                .codec_path(codec)
                .build(),
        );
    }

    let mut msg_svc = tonic_build::manual::Service::builder()
        .name("StellarGatewayMsg")
        .package("stellar.gateway.v1");
    for (name, route, input, output) in &msg_methods {
        msg_svc = msg_svc.method(
            tonic_build::manual::Method::builder()
                .name(name)
                .route_name(route)
                .input_type(input)
                .output_type(output)
                .codec_path(codec)
                .build(),
        );
    }

    tonic_build::manual::Builder::new()
        .build_server(false)
        .build_client(true)
        .compile(&[query_svc.build(), msg_svc.build()]);
}
