pub mod client;
pub mod common;
pub mod execution;
pub mod router;
pub mod storage;
pub mod token_transfer;
pub mod validation;

use crate::{event::IbcEvent, context::storage::SorobanStorage};

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
