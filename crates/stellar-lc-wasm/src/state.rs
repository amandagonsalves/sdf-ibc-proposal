use cw_storage_plus::{Item, Map};

use crate::types::{ClientState, ConsensusState};

pub const CLIENT_STATE: Item<ClientState> = Item::new("client_state");
pub const CONSENSUS_STATES: Map<u64, ConsensusState> = Map::new("consensus_states");
