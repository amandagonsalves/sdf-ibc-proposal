use crate::proto::{
    stellar_gateway_msg_server::{StellarGatewayMsg, StellarGatewayMsgServer},
    MsgAckPacketRequest, MsgAckPacketResponse, MsgCreateClientRequest, MsgCreateClientResponse,
    MsgRecvPacketRequest, MsgRecvPacketResponse, MsgRegisterCounterpartyRequest,
    MsgRegisterCounterpartyResponse, MsgSubmitMisbehaviourRequest, MsgSubmitMisbehaviourResponse,
    MsgTimeoutPacketRequest, MsgTimeoutPacketResponse, MsgUpdateClientRequest,
    MsgUpdateClientResponse, SubmitSignedTxRequest, SubmitSignedTxResponse,
};
use soroban_client::xdr::{Limits, ReadXdr, ScBytes, ScString, ScVal, ScVec, StringM, VecM, WriteXdr};
use stellar_ibc_core::api_client::ApiClient;
use stellar_ibc_core::rpc::SubmittedTx;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct MsgHandler {
    pub api: ApiClient,
}

impl MsgHandler {
    pub fn new(api: ApiClient) -> Self {
        Self { api }
    }

    pub fn into_server(self) -> StellarGatewayMsgServer<Self> {
        StellarGatewayMsgServer::new(self)
    }

    async fn prepare_router(
        &self,
        method: &str,
        args: Vec<ScVal>,
        signer: &str,
    ) -> Result<Vec<u8>, Status> {
        if signer.is_empty() {
            return Err(Status::invalid_argument(
                "signer (relayer source account) is required to prepare an unsigned tx",
            ));
        }
        tracing::info!(method, args = args.len(), %signer, "prepare_router via api");
        self.api
            .prepare_invoke(method, args, signer)
            .await
            .map_err(|error| {
                tracing::error!(%error, method, "prepare_invoke failed");
                Status::internal(format!("prepare_invoke({method}): {error}"))
            })
    }

    async fn invoke_router(&self, method: &str, args: Vec<ScVal>) -> Result<SubmittedTx, Status> {
        tracing::info!(method, args = args.len(), "invoke_router via api");
        match self.api.invoke_router(method, args).await {
            Ok(submitted) => {
                tracing::info!(
                    method,
                    hash = %submitted.hash,
                    has_return_value = submitted.return_value.is_some(),
                    "invoke_router ok"
                );
                Ok(submitted)
            }
            Err(error) => {
                tracing::error!(%error, method, "invoke_router failed");
                Err(Status::internal(format!(
                    "invoke_router({method}): {error}"
                )))
            }
        }
    }
}

fn scval_into_string(value: ScVal) -> Option<String> {
    match value {
        ScVal::String(ScString(sm)) => core::str::from_utf8(sm.as_slice())
            .ok()
            .map(|s| s.to_string()),
        ScVal::Symbol(sym) => core::str::from_utf8(sym.0.as_slice())
            .ok()
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn scval_string(s: &str) -> Result<ScVal, Status> {
    let sm = StringM::<{ u32::MAX }>::try_from(s.as_bytes())
        .map_err(|e| Status::invalid_argument(format!("invalid string for ScVal: {e}")))?;
    Ok(ScVal::String(ScString(sm)))
}

fn scval_bytes(b: &[u8]) -> Result<ScVal, Status> {
    let bm = b
        .to_vec()
        .try_into()
        .map_err(|e| Status::invalid_argument(format!("invalid bytes for ScVal: {e}")))?;
    Ok(ScVal::Bytes(ScBytes(bm)))
}

fn scval_u64(v: u64) -> ScVal {
    ScVal::U64(v)
}

fn scval_vec_of_bytes(items: &[Vec<u8>]) -> Result<ScVal, Status> {
    let inner: Result<Vec<ScVal>, Status> = items.iter().map(|b| scval_bytes(b)).collect();
    let vecm = VecM::<ScVal>::try_from(inner?)
        .map_err(|e| Status::invalid_argument(format!("invalid Vec<Bytes>: {e}")))?;
    Ok(ScVal::Vec(Some(ScVec(vecm))))
}

fn decode_packet_scval(bytes: &[u8]) -> Result<ScVal, Status> {
    ScVal::from_xdr(bytes, Limits::none())
        .map_err(|e| Status::invalid_argument(format!("packet ScVal XDR decode: {e}")))
}

#[tonic::async_trait]
impl StellarGatewayMsg for MsgHandler {
    #[tracing::instrument(skip(self, request), name = "grpc.submit_signed_tx")]
    async fn submit_signed_tx(
        &self,
        request: Request<SubmitSignedTxRequest>,
    ) -> Result<Response<SubmitSignedTxResponse>, Status> {
        let tx_xdr = request.into_inner().tx_xdr;
        tracing::info!(tx_bytes = tx_xdr.len(), "gRPC SubmitSignedTx");
        let submitted = self
            .api
            .submit_and_wait_for_result(&tx_xdr)
            .await
            .map_err(|error| {
                tracing::error!(%error, "submit_and_wait_for_result failed");
                Status::internal(format!("submit_and_wait: {error}"))
            })?;
        let return_value = submitted
            .return_value
            .and_then(|v| v.to_xdr(Limits::none()).ok())
            .unwrap_or_default();
        tracing::info!(tx_hash = %submitted.hash, "submit_signed_tx ok");
        Ok(Response::new(SubmitSignedTxResponse {
            tx_hash: submitted.hash,
            events: Vec::new(),
            return_value,
        }))
    }

    #[tracing::instrument(skip(self, request), name = "grpc.create_client")]
    async fn create_client(
        &self,
        request: Request<MsgCreateClientRequest>,
    ) -> Result<Response<MsgCreateClientResponse>, Status> {
        let req = request.into_inner();
        if req.client_type.is_empty() {
            return Err(Status::invalid_argument(
                "MsgCreateClientRequest.client_type is required",
            ));
        }
        tracing::info!(
            client_type = %req.client_type,
            client_state_bytes = req.client_state.len(),
            consensus_state_bytes = req.consensus_state.len(),
            height = req.height,
            "gRPC CreateClient"
        );
        let args = vec![
            scval_string(&req.client_type)?,
            scval_bytes(&req.client_state)?,
            scval_bytes(&req.consensus_state)?,
            scval_u64(req.height),
        ];
        let tx_xdr = self
            .prepare_router("create_client", args, &req.signer)
            .await?;
        tracing::info!(tx_bytes = tx_xdr.len(), "create_client prepared (unsigned)");
        Ok(Response::new(MsgCreateClientResponse {
            client_id: String::new(),
            tx_xdr,
        }))
    }

    #[tracing::instrument(skip(self, request), name = "grpc.update_client")]
    async fn update_client(
        &self,
        request: Request<MsgUpdateClientRequest>,
    ) -> Result<Response<MsgUpdateClientResponse>, Status> {
        let req = request.into_inner();
        if req.client_id.is_empty() {
            return Err(Status::invalid_argument(
                "MsgUpdateClientRequest.client_id is required",
            ));
        }
        tracing::info!(
            client_id = %req.client_id,
            header_bytes = req.header.len(),
            "gRPC UpdateClient"
        );
        let args = vec![scval_string(&req.client_id)?, scval_bytes(&req.header)?];
        let _ = self.invoke_router("update_client", args).await?;
        Ok(Response::new(MsgUpdateClientResponse {}))
    }

    #[tracing::instrument(skip(self, request), name = "grpc.register_counterparty")]
    async fn register_counterparty(
        &self,
        request: Request<MsgRegisterCounterpartyRequest>,
    ) -> Result<Response<MsgRegisterCounterpartyResponse>, Status> {
        let req = request.into_inner();
        if req.client_id.is_empty() || req.counterparty_client_id.is_empty() {
            return Err(Status::invalid_argument(
                "client_id and counterparty_client_id are required",
            ));
        }
        tracing::info!(
            client_id = %req.client_id,
            counterparty_client_id = %req.counterparty_client_id,
            prefix_segments = req.counterparty_commitment_prefix.len(),
            "gRPC RegisterCounterparty"
        );
        let args = vec![
            scval_string(&req.client_id)?,
            scval_string(&req.counterparty_client_id)?,
            scval_vec_of_bytes(&req.counterparty_commitment_prefix)?,
        ];
        let _ = self.invoke_router("register_counterparty", args).await?;
        Ok(Response::new(MsgRegisterCounterpartyResponse {}))
    }

    #[tracing::instrument(skip(self, request), name = "grpc.recv_packet")]
    async fn recv_packet(
        &self,
        request: Request<MsgRecvPacketRequest>,
    ) -> Result<Response<MsgRecvPacketResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            packet_bytes = req.packet.len(),
            proof_bytes = req.proof.len(),
            proof_height = req.proof_height,
            "gRPC RecvPacket"
        );
        let args = vec![
            decode_packet_scval(&req.packet)?,
            scval_bytes(&req.proof)?,
            scval_u64(req.proof_height),
        ];
        let _ = self.invoke_router("recv_packet", args).await?;
        Ok(Response::new(MsgRecvPacketResponse {}))
    }

    #[tracing::instrument(skip(self, request), name = "grpc.ack_packet")]
    async fn ack_packet(
        &self,
        request: Request<MsgAckPacketRequest>,
    ) -> Result<Response<MsgAckPacketResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            packet_bytes = req.packet.len(),
            ack_bytes = req.acknowledgement.len(),
            proof_bytes = req.proof.len(),
            proof_height = req.proof_height,
            "gRPC AckPacket"
        );
        let acks = scval_vec_of_bytes(&[req.acknowledgement])?;
        let args = vec![
            decode_packet_scval(&req.packet)?,
            acks,
            scval_bytes(&req.proof)?,
            scval_u64(req.proof_height),
        ];
        let _ = self.invoke_router("acknowledge_packet", args).await?;
        Ok(Response::new(MsgAckPacketResponse {}))
    }

    #[tracing::instrument(skip(self, request), name = "grpc.timeout_packet")]
    async fn timeout_packet(
        &self,
        request: Request<MsgTimeoutPacketRequest>,
    ) -> Result<Response<MsgTimeoutPacketResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            packet_bytes = req.packet.len(),
            proof_bytes = req.proof.len(),
            proof_height = req.proof_height,
            "gRPC TimeoutPacket"
        );
        let args = vec![
            decode_packet_scval(&req.packet)?,
            scval_bytes(&req.proof)?,
            scval_u64(req.proof_height),
        ];
        let _ = self.invoke_router("timeout_packet", args).await?;
        Ok(Response::new(MsgTimeoutPacketResponse {}))
    }

    #[tracing::instrument(skip(self, request), name = "grpc.submit_misbehaviour")]
    async fn submit_misbehaviour(
        &self,
        request: Request<MsgSubmitMisbehaviourRequest>,
    ) -> Result<Response<MsgSubmitMisbehaviourResponse>, Status> {
        let req = request.into_inner();
        if req.client_id.is_empty() {
            return Err(Status::invalid_argument(
                "MsgSubmitMisbehaviourRequest.client_id is required",
            ));
        }
        tracing::info!(
            client_id = %req.client_id,
            client_message_bytes = req.client_message.len(),
            "gRPC SubmitMisbehaviour"
        );
        let args = vec![
            scval_string(&req.client_id)?,
            scval_bytes(&req.client_message)?,
        ];
        let _ = self.invoke_router("update_client", args).await?;
        Ok(Response::new(MsgSubmitMisbehaviourResponse {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handler() -> MsgHandler {
        MsgHandler::new(ApiClient::new("http://127.0.0.1:8101"))
    }

    #[tokio::test]
    async fn submit_misbehaviour_rejects_missing_client_id() {
        let h = handler();
        let req = Request::new(MsgSubmitMisbehaviourRequest {
            client_id: String::new(),
            client_message: vec![1, 2, 3],
            signer: String::new(),
        });
        let err = h.submit_misbehaviour(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
        assert!(err.message().contains("client_id"));
    }

    #[test]
    fn scval_helpers_produce_expected_variants() {
        let s = scval_string("transfer").unwrap();
        assert!(matches!(s, ScVal::String(_)));

        let b = scval_bytes(b"abc").unwrap();
        assert!(matches!(b, ScVal::Bytes(_)));

        let u = scval_u64(42);
        assert!(matches!(u, ScVal::U64(42)));

        let v = scval_vec_of_bytes(&[b"ibc".to_vec(), b"\x01\x02".to_vec()]).unwrap();
        let inner = match v {
            ScVal::Vec(Some(ScVec(items))) => items,
            _ => panic!("expected ScVal::Vec(Some(_))"),
        };
        assert_eq!(inner.len(), 2);
        assert!(matches!(inner[0], ScVal::Bytes(_)));
    }
}
