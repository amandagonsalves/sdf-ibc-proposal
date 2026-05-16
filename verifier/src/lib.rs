#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;

use stellar_ibc_types::{StellarClientState, StellarConsensusState, StellarHeader};

#[derive(Debug)]
pub enum VerifyError {
    SequenceNotMonotonic { trusted: u64, got: u64 },
    TimestampNotMonotonic { trusted: u64, got: u64 },
    ClockDriftExceeded,
    LedgerHashMismatch,
    InsufficientSignatures { got: usize, required: usize },
    XdrDecodeError(String),
}

pub fn verify_header(
    _trusted_cs: &StellarConsensusState,
    _header: &StellarHeader,
    _client_state: &StellarClientState,
    _now_secs: u64,
) -> Result<(), VerifyError> {
    Ok(())
}

pub fn check_misbehaviour(h1: &StellarHeader, h2: &StellarHeader) -> bool {
    let _ = (h1, h2);
    false
}

#[cfg(test)]
mod tests {
    use ibc_core::client::types::Height;
    use stellar_ibc_types::{StellarClientState, StellarConsensusState, StellarHeader};

    use super::*;

    fn dummy_cs() -> StellarConsensusState {
        StellarConsensusState::new(1_700_000_000, [0u8; 32], [0u8; 32])
    }

    fn dummy_client_state() -> StellarClientState {
        StellarClientState {
            chain_id: "stellar-testnet".into(),
            latest_height: Height::new(0, 1).unwrap(),
            trusting_period_secs: 3600,
            max_clock_drift_secs: 60,
            frozen_height: None,
            known_validators: alloc::vec![[0u8; 32]],
            trust_threshold: (2, 3),
        }
    }

    fn dummy_header() -> StellarHeader {
        StellarHeader {
            ledger_header_xdr: alloc::vec![],
            scp_envelopes: alloc::vec![],
            trusted_height: Height::new(0, 1).unwrap(),
            trusted_validators: alloc::vec![[0u8; 32]],
        }
    }

    #[test]
    fn verify_header_stub_returns_ok() {
        let result = verify_header(&dummy_cs(), &dummy_header(), &dummy_client_state(), 0);
        assert!(result.is_ok());
    }

    #[test]
    fn check_misbehaviour_stub_returns_false() {
        let h = dummy_header();
        assert!(!check_misbehaviour(&h, &h));
    }
}
