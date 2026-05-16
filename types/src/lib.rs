#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod client_state;
mod consensus_state;
mod header;

pub use client_state::StellarClientState;
pub use consensus_state::StellarConsensusState;
pub use header::{ScpEnvelope, StellarHeader};

pub const CLIENT_TYPE: &str = "10-stellar";
