use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub counter: u64,
    pub minimal_donation: Coin,
}

pub const STATE: Item<State> = Item::new("state");
pub const OWNER: Item<Addr> = Item::new("owner");
