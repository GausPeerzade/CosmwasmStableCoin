use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const STABLE: Item<Addr> = Item::new("stabletoken");
pub const COLLATERALDEPOSITED: Map<Addr, Uint128> = Map::new("collateradeposited");
pub const TOKENSMINTED: Map<Addr, Uint128> = Map::new("tokensminted");
pub const LIQUIDATIONTH: Item<Uint128> = Item::new("liquidationThreashold");
