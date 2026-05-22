include!(concat!(env!("PROTOS_OUT_DIR"), "/stellar.gateway.v1.rs"));

include!(concat!(
    env!("PROTOS_OUT_DIR"),
    "/stellar.gateway.v1.StellarGatewayQuery.rs"
));

include!(concat!(
    env!("PROTOS_OUT_DIR"),
    "/stellar.gateway.v1.StellarGatewayMsg.rs"
));
