#![cfg(test)]

use super::*;
use soroban_mock_lc::{MockLightClient, MockLightClientClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Bytes, Env, String};

fn setup() -> (Env, Address, IbcRouterClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let router_id = env.register(IbcRouter, (admin.clone(),));
    let router = IbcRouterClient::new(&env, &router_id);

    let lc_id = env.register(MockLightClient, ());

    router.register_client_type(&String::from_str(&env, "mock"), &lc_id);
    (env, router_id, router, lc_id)
}

#[test]
fn register_client_type_stores_lc_address() {
    let (env, _router_id, router, lc_id) = setup();
    let resolved = router.lc_address(&String::from_str(&env, "mock")).unwrap();
    assert_eq!(resolved, lc_id);
}

#[test]
fn create_client_initialises_lc_and_returns_unique_id() {
    let (env, _router_id, router, lc_id) = setup();

    let cs = Bytes::from_slice(&env, b"client-state");
    let cons = Bytes::from_slice(&env, b"consensus-state");
    let id1 = router.create_client(&String::from_str(&env, "mock"), &cs, &cons, &1);
    let id2 = router.create_client(&String::from_str(&env, "mock"), &cs, &cons, &1);

    assert_ne!(id1, id2);
    assert_eq!(id1, String::from_str(&env, "mock-0"));
    assert_eq!(id2, String::from_str(&env, "mock-1"));

    let lc = MockLightClientClient::new(&env, &lc_id);
    assert_eq!(lc.latest_height(&id1), 1);
    assert_eq!(lc.client_state(&id1), cs);
}

#[test]
fn register_counterparty_stores_mapping() {
    let (env, _router_id, router, _lc_id) = setup();

    let id = router.create_client(
        &String::from_str(&env, "mock"),
        &Bytes::from_slice(&env, b"cs"),
        &Bytes::from_slice(&env, b"cons"),
        &1,
    );

    let counterparty_id = String::from_str(&env, "07-tendermint-0");
    let prefix = vec![&env, Bytes::from_slice(&env, b"ibc")];
    router.register_counterparty(&id, &counterparty_id, &prefix);

    let cp = router.counterparty(&id).unwrap();
    assert_eq!(cp.client_id, counterparty_id);
    assert_eq!(cp.commitment_prefix.len(), 1);
}

#[test]
#[should_panic(expected = "counterparty already registered")]
fn register_counterparty_rejects_duplicate() {
    let (env, _router_id, router, _lc_id) = setup();
    let id = router.create_client(
        &String::from_str(&env, "mock"),
        &Bytes::from_slice(&env, b"cs"),
        &Bytes::from_slice(&env, b"cons"),
        &1,
    );
    let cp_id = String::from_str(&env, "07-tendermint-0");
    let prefix = vec![&env, Bytes::from_slice(&env, b"ibc")];
    router.register_counterparty(&id, &cp_id, &prefix);
    router.register_counterparty(&id, &cp_id, &prefix);
}

#[test]
#[should_panic(expected = "client_id not found")]
fn register_counterparty_rejects_unknown_client() {
    let (env, _router_id, router, _lc_id) = setup();
    router.register_counterparty(
        &String::from_str(&env, "mock-999"),
        &String::from_str(&env, "07-tendermint-0"),
        &vec![&env, Bytes::from_slice(&env, b"ibc")],
    );
}

#[test]
fn update_client_bumps_height_via_lc() {
    let (env, _router_id, router, lc_id) = setup();
    let id = router.create_client(
        &String::from_str(&env, "mock"),
        &Bytes::from_slice(&env, b"cs"),
        &Bytes::from_slice(&env, b"cons"),
        &5,
    );

    let new_h = router.update_client(&id, &Bytes::from_slice(&env, b"msg"));
    assert_eq!(new_h, 6);
    assert_eq!(MockLightClientClient::new(&env, &lc_id).latest_height(&id), 6);
    assert!(!router.frozen(&id));
}
