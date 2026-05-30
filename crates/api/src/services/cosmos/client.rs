use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use cosmrs::crypto::secp256k1::SigningKey;
use cosmrs::proto::cosmos::bank::v1beta1::MsgSend;
use cosmrs::proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use cosmrs::proto::cosmos::gov::v1::{MsgSubmitProposal, MsgVote};
use cosmrs::tendermint::chain::Id as ChainId;
use cosmrs::tx::{Body, Fee, SignDoc, SignerInfo};
use cosmrs::{Any, Coin};
use ibc_proto::ibc::lightclients::wasm::v1::MsgStoreCode;
use prost::Message;
use reqwest::Client as HttpClient;
use serde_json::Value;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

use crate::config::CosmosConfig;

pub struct CosmosClient {
    pub config: CosmosConfig,
    http: HttpClient,
    proposer_key: Option<SigningKey>,
    proposer_address: Option<String>,
    funder_key: Option<SigningKey>,
    funder_address: Option<String>,
}

fn load_key(hex_str: &str, prefix: &str, label: &str) -> Result<Option<(SigningKey, String)>> {
    let trimmed = hex_str.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let bytes = hex::decode(trimmed)
        .with_context(|| format!("{label} is not valid hex"))?;
    let key = SigningKey::from_slice(&bytes)
        .map_err(|e| anyhow!("invalid {label} secp256k1 key: {e}"))?;
    let addr = key
        .public_key()
        .account_id(prefix)
        .map_err(|e| anyhow!("derive {label} account id: {e}"))?
        .to_string();
    Ok(Some((key, addr)))
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub address: String,
    pub account_number: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone)]
pub struct BroadcastResult {
    pub tx_hash: String,
    pub code: u32,
    pub raw_log: String,
}

impl CosmosClient {
    pub fn new(config: CosmosConfig) -> Result<Self> {
        let (proposer_key, proposer_address) = load_key(
            &config.proposer_private_key_hex,
            &config.account_prefix,
            "COSMOS_PROPOSER_PRIVATE_KEY",
        )?
        .map(|(k, a)| (Some(k), Some(a)))
        .unwrap_or((None, None));

        let (funder_key, funder_address) = load_key(
            &config.funder_private_key_hex,
            &config.account_prefix,
            "COSMOS_FUNDER_PRIVATE_KEY",
        )?
        .map(|(k, a)| (Some(k), Some(a)))
        .unwrap_or((None, None));

        Ok(Self {
            config,
            http: HttpClient::new(),
            proposer_key,
            proposer_address,
            funder_key,
            funder_address,
        })
    }

    pub fn proposer_address(&self) -> Option<&str> {
        self.proposer_address.as_deref()
    }

    pub fn funder_address(&self) -> Option<&str> {
        self.funder_address.as_deref()
    }

    fn rest(&self, path: &str) -> String {
        format!("{}{}", self.config.rest_url.trim_end_matches('/'), path)
    }

    pub async fn node_info(&self) -> Result<Value> {
        let url = self.rest("/cosmos/base/tendermint/v1beta1/node_info");
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn proposals_by_status(&self, status: &str) -> Result<Value> {
        let url = self.rest(&format!(
            "/cosmos/gov/v1/proposals?proposal_status={}",
            status
        ));
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn proposal(&self, proposal_id: u64) -> Result<Value> {
        let url = self.rest(&format!("/cosmos/gov/v1/proposals/{}", proposal_id));
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn gov_deposit_params(&self) -> Result<Value> {
        let url = self.rest("/cosmos/gov/v1/params/deposit");
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn tx_by_hash(&self, tx_hash: &str) -> Result<Value> {
        let url = self.rest(&format!("/cosmos/tx/v1beta1/txs/{}", tx_hash));
        let resp = self.http.get(&url).send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow!("tx not found"));
        }
        Ok(resp.error_for_status()?.json().await?)
    }

    pub async fn ibc_wasm_checksums(&self) -> Result<Value> {
        let url = self.rest("/ibc/lightclients/wasm/v1/checksums");
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn account_exists(&self, address: &str) -> Result<bool> {
        let url = self.rest(&format!("/cosmos/auth/v1beta1/accounts/{}", address));
        let resp = self.http.get(&url).send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(false);
        }
        resp.error_for_status()?;
        Ok(true)
    }

    pub async fn account_info(&self, address: &str) -> Result<AccountInfo> {
        let url = self.rest(&format!("/cosmos/auth/v1beta1/accounts/{}", address));
        let body: Value = self.http.get(&url).send().await?.error_for_status()?.json().await?;
        let account = body
            .get("account")
            .ok_or_else(|| anyhow!("response missing 'account': {body}"))?;
        let account_number = account
            .get("account_number")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| anyhow!("account_number missing: {account}"))?;
        let sequence = account
            .get("sequence")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        Ok(AccountInfo {
            address: address.to_string(),
            account_number,
            sequence,
        })
    }

    pub async fn gov_module_address(&self) -> Result<String> {
        let url = self.rest("/cosmos/auth/v1beta1/module_accounts/gov");
        let body: Value = self.http.get(&url).send().await?.error_for_status()?.json().await?;
        body.pointer("/account/value/address")
            .or_else(|| body.pointer("/account/base_account/address"))
            .or_else(|| body.pointer("/account/address"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("gov module address not found in response: {body}"))
    }

    fn proposer_pair(&self) -> Result<(&SigningKey, &str)> {
        match (&self.proposer_key, &self.proposer_address) {
            (Some(k), Some(a)) => Ok((k, a.as_str())),
            _ => Err(anyhow!("COSMOS_PROPOSER_PRIVATE_KEY not configured")),
        }
    }

    fn funder_pair(&self) -> Result<(&SigningKey, &str)> {
        match (&self.funder_key, &self.funder_address) {
            (Some(k), Some(a)) => Ok((k, a.as_str())),
            _ => Err(anyhow!("COSMOS_FUNDER_PRIVATE_KEY not configured")),
        }
    }

    async fn sign_and_broadcast(
        &self,
        signing_key: &SigningKey,
        from_address: &str,
        msg_any: Any,
        memo: &str,
        gas_limit: u64,
        fee_amount: u128,
    ) -> Result<BroadcastResult> {
        let account = self.account_info(from_address).await?;

        let body = Body::new(vec![msg_any], memo, 0u32);
        let fee = Fee::from_amount_and_gas(
            Coin {
                denom: cosmrs::Denom::from_str(&self.config.gas_denom)
                    .map_err(|e| anyhow!("invalid gas denom: {e}"))?,
                amount: fee_amount,
            },
            gas_limit,
        );
        let auth_info =
            SignerInfo::single_direct(Some(signing_key.public_key()), account.sequence)
                .auth_info(fee);
        let chain_id = ChainId::try_from(self.config.chain_id.clone())
            .map_err(|e| anyhow!("invalid chain id: {e}"))?;
        let sign_doc = SignDoc::new(&body, &auth_info, &chain_id, account.account_number)
            .map_err(|e| anyhow!("build sign doc: {e}"))?;
        let tx_raw_bytes = sign_doc
            .sign(signing_key)
            .map_err(|e| anyhow!("sign tx: {e}"))?
            .to_bytes()
            .map_err(|e| anyhow!("encode tx_raw: {e}"))?;

        self.broadcast_tx_bytes(tx_raw_bytes).await
    }

    async fn broadcast_tx_bytes(&self, tx_bytes: Vec<u8>) -> Result<BroadcastResult> {
        let url = self.rest("/cosmos/tx/v1beta1/txs");
        let body = serde_json::json!({
            "tx_bytes": BASE64.encode(&tx_bytes),
            "mode": "BROADCAST_MODE_SYNC",
        });
        let resp = self.http.post(&url).json(&body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(anyhow!("broadcast failed ({status}): {text}"));
        }
        let value: Value = serde_json::from_str(&text)
            .with_context(|| format!("non-json broadcast response: {text}"))?;
        let tx_response = value
            .get("tx_response")
            .ok_or_else(|| anyhow!("missing tx_response: {value}"))?;
        let tx_hash = tx_response
            .get("txhash")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let code = tx_response
            .get("code")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let raw_log = tx_response
            .get("raw_log")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        Ok(BroadcastResult {
            tx_hash,
            code,
            raw_log,
        })
    }

    pub async fn wait_for_tx(&self, tx_hash: &str, timeout: Duration) -> Result<Value> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut delay = Duration::from_millis(500);
        loop {
            match self.tx_by_hash(tx_hash).await {
                Ok(v) => return Ok(v),
                Err(_) if tokio::time::Instant::now() < deadline => {
                    sleep(delay).await;
                    if delay < Duration::from_secs(2) {
                        delay *= 2;
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub async fn submit_store_code_proposal(
        &self,
        wasm_bytes: Vec<u8>,
        title: String,
        summary: String,
        deposit_amount: u128,
        gas_limit: u64,
        fee_amount: u128,
    ) -> Result<BroadcastResult> {
        let gov_addr = self.gov_module_address().await?;
        let (signing_key, proposer) = self.proposer_pair()?;
        let proposer = proposer.to_string();

        let store_code = MsgStoreCode {
            signer: gov_addr,
            wasm_byte_code: wasm_bytes,
        };
        let store_any = Any {
            type_url: "/ibc.lightclients.wasm.v1.MsgStoreCode".to_string(),
            value: store_code.encode_to_vec(),
        };

        let msg = MsgSubmitProposal {
            messages: vec![cosmrs::proto::Any {
                type_url: store_any.type_url.clone(),
                value: store_any.value.clone(),
            }],
            initial_deposit: vec![ProtoCoin {
                denom: self.config.gas_denom.clone(),
                amount: deposit_amount.to_string(),
            }],
            proposer: proposer.clone(),
            metadata: String::new(),
            title,
            summary,
            expedited: false,
        };

        let msg_any = Any {
            type_url: "/cosmos.gov.v1.MsgSubmitProposal".to_string(),
            value: msg.encode_to_vec(),
        };

        self.sign_and_broadcast(signing_key, &proposer, msg_any, "upload-lc-wasm", gas_limit, fee_amount)
            .await
    }

    pub async fn submit_vote(
        &self,
        proposal_id: u64,
        option: i32,
        gas_limit: u64,
        fee_amount: u128,
    ) -> Result<BroadcastResult> {
        let (signing_key, voter) = self
            .funder_pair()
            .or_else(|_| self.proposer_pair())?;
        let voter = voter.to_string();
        let msg = MsgVote {
            proposal_id,
            voter: voter.clone(),
            option,
            metadata: String::new(),
        };
        let msg_any = Any {
            type_url: "/cosmos.gov.v1.MsgVote".to_string(),
            value: msg.encode_to_vec(),
        };
        self.sign_and_broadcast(signing_key, &voter, msg_any, "upload-lc-wasm-vote", gas_limit, fee_amount)
            .await
    }

    pub async fn submit_bank_send(
        &self,
        to_address: String,
        amount: u128,
        gas_limit: u64,
        fee_amount: u128,
    ) -> Result<BroadcastResult> {
        let (signing_key, funder) = self.funder_pair()?;
        let funder = funder.to_string();
        let msg = MsgSend {
            from_address: funder.clone(),
            to_address,
            amount: vec![ProtoCoin {
                denom: self.config.gas_denom.clone(),
                amount: amount.to_string(),
            }],
        };
        let msg_any = Any {
            type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
            value: msg.encode_to_vec(),
        };
        self.sign_and_broadcast(signing_key, &funder, msg_any, "bank-send", gas_limit, fee_amount)
            .await
    }

    pub fn extract_proposal_id(tx_response: &Value) -> Option<u64> {
        let events = tx_response.pointer("/tx_response/events").or_else(|| tx_response.get("events"))?;
        for event in events.as_array()? {
            let kind = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if kind == "submit_proposal" {
                for attr in event.get("attributes")?.as_array()? {
                    let key = attr.get("key").and_then(|v| v.as_str()).unwrap_or("");
                    if key == "proposal_id" {
                        if let Some(value) = attr.get("value").and_then(|v| v.as_str()) {
                            if let Ok(id) = value.parse::<u64>() {
                                return Some(id);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

