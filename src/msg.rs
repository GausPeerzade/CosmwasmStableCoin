use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub oracle: Addr,
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
    CustomMsg { val: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CustomResponse {
    val: String,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub struct InfoResponse {
//     collateral_deposited: Uint128,
//     total_debt: Uint128,
//     liquidity_threshold: Uint128,
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
