use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
    pub oracle: String,
    pub denom: String,
    pub min_threashold: Uint128,
    pub liquidity_threashold: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetToken {
        token: Addr,
    },
    DepositCollateral {},
    DepositCollateralAndMint {
        token_amount: Uint128,
    },
    RedeemCollateral {
        amount: Uint128,
    },
    RedeemCollateralAndBurn {
        amount_collateral: Uint128,
        amount_token: Uint128,
    },

    Liquidate {
        user: Addr,
        amount_token: Uint128,
    },
    Swap {
        amount_token: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Info { user: Addr },
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CustomResponse {
    val: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InfoResponse {
    pub collateral_deposited: Uint128,
    pub total_debt: Uint128,
    pub health_factor: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub total_collateral: Uint128,
    pub oracle_price: Uint128,
    pub fees: Uint128,
    pub liquidity_threashold: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
