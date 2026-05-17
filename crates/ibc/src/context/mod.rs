pub mod client;
pub mod common;
pub mod execution;
pub mod router;
pub mod storage;
pub mod token_transfer;
pub mod validation;

use crate::{context::storage::SorobanStorage, event::IbcEvent};

pub struct StellarIbcContext<S> {
    pub store: S,
    pub events: Vec<IbcEvent>,
}

impl<S: SorobanStorage> StellarIbcContext<S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            events: Vec::new(),
        }
    }
}
