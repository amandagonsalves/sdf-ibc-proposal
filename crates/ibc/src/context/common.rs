use ibc::core::{
    client::types::Height,
    host::types::identifiers::ClientId,
};

use crate::{
    context::{StellarIbcContext, storage::SorobanStorage},
    error::Error,
};

pub trait IbcCommonContext {
    fn client_state(&self, client_id: &ClientId) -> Result<Vec<u8>, Error>;
    fn store_client_state(
        &mut self,
        client_id: &ClientId,
        client_state: Vec<u8>,
    ) -> Result<(), Error>;

    fn consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Vec<u8>, Error>;
    fn store_consensus_state(
        &mut self,
        client_id: &ClientId,
        height: &Height,
        consensus_state: Vec<u8>,
    ) -> Result<(), Error>;

    fn client_counter(&self) -> Result<u64, Error>;
    fn increase_client_counter(&mut self) -> Result<(), Error>;
}

impl<S: SorobanStorage> IbcCommonContext for StellarIbcContext<S> {
    fn client_state(&self, _client_id: &ClientId) -> Result<Vec<u8>, Error> {
        Err(Error::Storage("not implemented".into()))
    }

    fn store_client_state(
        &mut self,
        _client_id: &ClientId,
        _client_state: Vec<u8>,
    ) -> Result<(), Error> {
        Err(Error::Storage("not implemented".into()))
    }

    fn consensus_state(
        &self,
        _client_id: &ClientId,
        _height: &Height,
    ) -> Result<Vec<u8>, Error> {
        Err(Error::Storage("not implemented".into()))
    }

    fn store_consensus_state(
        &mut self,
        _client_id: &ClientId,
        _height: &Height,
        _consensus_state: Vec<u8>,
    ) -> Result<(), Error> {
        Err(Error::Storage("not implemented".into()))
    }

    fn client_counter(&self) -> Result<u64, Error> {
        Err(Error::Storage("not implemented".into()))
    }

    fn increase_client_counter(&mut self) -> Result<(), Error> {
        Err(Error::Storage("not implemented".into()))
    }
}
