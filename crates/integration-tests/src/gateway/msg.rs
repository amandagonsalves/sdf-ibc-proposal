use soroban_client::xdr::ScVal;
use tonic::Code;

use stellar_hermes_gateway::proto::{
    MsgAckPacketRequest, MsgCreateClientRequest, MsgRecvPacketRequest,
    MsgRegisterCounterpartyRequest, MsgSubmitMisbehaviourRequest, MsgTimeoutPacketRequest,
    MsgUpdateClientRequest, SubmitSignedTxRequest,
};
use stellar_ibc_core::conversion::{scval_from_xdr, scval_to_xdr, scval_vec_of_bytes};

use super::mock::{sample_packet, GatewayTest};

#[tokio::test]
async fn create_client_rejects_empty_client_type() {
    let t = GatewayTest::start(None).await;
    let err = t
        .msg()
        .create_client(MsgCreateClientRequest {
            client_state: vec![],
            consensus_state: vec![],
            client_type: String::new(),
            height: 0,
            signer: "GSIGNER".into(),
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("client_type"));
}

#[tokio::test]
async fn update_client_rejects_empty_client_id() {
    let t = GatewayTest::start(None).await;
    let err = t
        .msg()
        .update_client(MsgUpdateClientRequest {
            client_id: String::new(),
            header: vec![],
            signer: "GSIGNER".into(),
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("client_id"));
}

#[tokio::test]
async fn register_counterparty_rejects_empty_ids() {
    let t = GatewayTest::start(None).await;
    let err = t
        .msg()
        .register_counterparty(MsgRegisterCounterpartyRequest {
            client_id: String::new(),
            counterparty_client_id: String::new(),
            counterparty_commitment_prefix: vec![],
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::InvalidArgument);
}

#[tokio::test]
async fn submit_misbehaviour_rejects_empty_client_id() {
    let t = GatewayTest::start(None).await;
    let err = t
        .msg()
        .submit_misbehaviour(MsgSubmitMisbehaviourRequest {
            client_id: String::new(),
            client_message: vec![1, 2, 3],
            signer: "GSIGNER".into(),
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("client_id"));
}

#[tokio::test]
async fn recv_packet_forwards_packet_proof_and_height() {
    let t = GatewayTest::start(None).await;
    t.with_data(|d| d.prepare_tx_xdr = b"unsigned-recv".to_vec());

    let packet = sample_packet();
    let resp = t
        .msg()
        .recv_packet(MsgRecvPacketRequest {
            packet: scval_to_xdr(&packet).unwrap(),
            proof: vec![9, 9, 9],
            proof_height: 42,
            signer: "GSIGNER".into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.tx_xdr, b"unsigned-recv");

    t.with_data(|d| {
        let call = d.prepare_calls.last().expect("prepare called");
        assert_eq!(call.method, "recv_packet");
        assert_eq!(call.signer, "GSIGNER");
        assert_eq!(call.args.len(), 3);
        assert_eq!(scval_from_xdr(&call.args[0]).unwrap(), packet);
        assert_eq!(scval_from_xdr(&call.args[2]).unwrap(), ScVal::U64(42));
    });
}

#[tokio::test]
async fn ack_packet_wraps_acknowledgement_as_vec_of_bytes() {
    let t = GatewayTest::start(None).await;
    t.with_data(|d| d.prepare_tx_xdr = b"unsigned-ack".to_vec());

    let packet = sample_packet();
    let ack = vec![0xAA, 0xBB];
    let resp = t
        .msg()
        .ack_packet(MsgAckPacketRequest {
            packet: scval_to_xdr(&packet).unwrap(),
            acknowledgement: ack.clone(),
            proof: vec![1],
            proof_height: 7,
            signer: "GSIGNER".into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.tx_xdr, b"unsigned-ack");

    t.with_data(|d| {
        let call = d.prepare_calls.last().expect("prepare called");
        assert_eq!(call.method, "acknowledge_packet");
        assert_eq!(call.args.len(), 4);
        assert_eq!(scval_from_xdr(&call.args[0]).unwrap(), packet);
        assert_eq!(
            scval_from_xdr(&call.args[1]).unwrap(),
            scval_vec_of_bytes(&[ack]).unwrap()
        );
        assert_eq!(scval_from_xdr(&call.args[3]).unwrap(), ScVal::U64(7));
    });
}

#[tokio::test]
async fn timeout_packet_forwards_packet_and_height() {
    let t = GatewayTest::start(None).await;
    t.with_data(|d| d.prepare_tx_xdr = b"unsigned-timeout".to_vec());

    let packet = sample_packet();
    let resp = t
        .msg()
        .timeout_packet(MsgTimeoutPacketRequest {
            packet: scval_to_xdr(&packet).unwrap(),
            proof: vec![5],
            proof_height: 99,
            signer: "GSIGNER".into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.tx_xdr, b"unsigned-timeout");

    t.with_data(|d| {
        let call = d.prepare_calls.last().expect("prepare called");
        assert_eq!(call.method, "timeout_packet");
        assert_eq!(call.args.len(), 3);
        assert_eq!(scval_from_xdr(&call.args[0]).unwrap(), packet);
        assert_eq!(scval_from_xdr(&call.args[2]).unwrap(), ScVal::U64(99));
    });
}

#[tokio::test]
async fn submit_signed_tx_returns_hash_from_api() {
    let t = GatewayTest::start(None).await;
    t.with_data(|d| d.submit_hash = "abc123".to_string());

    let resp = t
        .msg()
        .submit_signed_tx(SubmitSignedTxRequest {
            tx_xdr: b"signed-bytes".to_vec(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.tx_hash, "abc123");

    t.with_data(|d| {
        assert_eq!(d.submit_calls.last().unwrap(), &hex::encode(b"signed-bytes"));
    });
}
