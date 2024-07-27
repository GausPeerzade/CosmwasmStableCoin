use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub oracle: Addr,
    pub denom: String,
    pub min_threashold: Uint128,
    pub liquidity_threashold: Uint128,
    pub token_set : bool
}

pub const STATE: Item<State> = Item::new("state");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const STABLE: Item<Addr> = Item::new("stabletoken");
pub const COLLATERALDEPOSITED: Map<Addr, Uint128> = Map::new("collateradeposited");
pub const TOKENSMINTED: Map<Addr, Uint128> = Map::new("tokensminted");
pub const LIQUIDATIONTH: Item<Uint128> = Item::new("liquidationThreashold");
pub const CONFIG: Item<Config> = Item::new("config");
