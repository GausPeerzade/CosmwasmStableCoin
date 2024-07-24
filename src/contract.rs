#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    coin, entry_point, to_json_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, SubMsg, WasmMsg,
};
use cosmwasm_std::{Addr, BankMsg, Uint128};
use cw2::set_contract_version;

use cw0::parse_reply_instantiate_data;
use cw20::Denom::Cw20;
use cw20::{Cw20ExecuteMsg, Denom, Expiration, MinterResponse};
use cw20_base::contract::query_balance;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{COLLATERALDEPOSITED, LIQUIDATIONTH, OWNER, STABLE, TOKENSMINTED};

const CONTRACT_NAME: &str = "crates.io:cw-stablecoin";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = msg.owner.unwrap();
    let validate_owner = deps.api.addr_validate(&owner)?;

    OWNER.save(deps.storage, &validate_owner)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetConfig { token, threashold } => execute_set_config(deps, token, threashold),
        ExecuteMsg::DepositCollateral {} => execute_deposit_collateral(deps, info),
        ExecuteMsg::DepositCollateralAndMint { token_amount } => {
            execute_deposit_collateral_mint(deps, info, token_amount)
        }
        ExecuteMsg::RedeemCollateral { amount } => {
            execute_redeem_collateral(deps, env, info, amount)
        }
        ExecuteMsg::RedeemCollateralAndBurn {
            amount_collateral,
            amount_token,
        } => execute_redeem_collateral_burn(deps, env, info, amount_collateral, amount_token),
        ExecuteMsg::Liquidate { user, amount_token } => {
            execute_liquidation(deps, env, info, user, amount_token)
        }
    }
}

fn execute_set_config(
    deps: DepsMut,
    token: Addr,
    threashold: Uint128,
) -> Result<Response, ContractError> {
    STABLE.save(deps.storage, &token)?;
    LIQUIDATIONTH.save(deps.storage, &threashold)?;
    Ok(Response::new())
}

fn execute_deposit_collateral(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let sent_funds = info.funds;

    let user = info.sender;

    let amount_sent = amount_sent(sent_funds);
    println!("amount is {}", amount_sent);
    let user_deposit = COLLATERALDEPOSITED
        .load(deps.storage, user.clone())
        .unwrap_or_default();

    println!("user deposit is {}", user_deposit);

    COLLATERALDEPOSITED.save(deps.storage, user.clone(), &(amount_sent + user_deposit))?;
    println!("Deposit total is in ex {}", amount_sent + user_deposit);
    Ok(Response::new().add_attribute("execute deposit", "collateral deposited"))
}

fn execute_deposit_collateral_mint(
    deps: DepsMut,
    info: MessageInfo,
    token_amount: Uint128,
) -> Result<Response, ContractError> {
    let sent_funds = info.funds;

    let user = info.sender;

    let amount_sent = amount_sent(sent_funds);
    let user_deposit = COLLATERALDEPOSITED
        .load(deps.storage, user.clone())
        .unwrap_or_default();

    let mut token_minted = TOKENSMINTED
        .load(deps.storage, user.clone())
        .unwrap_or_default();

    token_minted += token_amount;
    TOKENSMINTED.save(deps.storage, user.clone(), &token_minted)?;

    COLLATERALDEPOSITED.save(deps.storage, user.clone(), &(amount_sent + user_deposit))?;

    println!("Deposit total is in ex mint{}", amount_sent + user_deposit);

    let collateral = COLLATERALDEPOSITED.load(deps.storage, user.clone())?;
    let collateral_value = calculate_collateral_usd(collateral);
    let liquidity_threashold = LIQUIDATIONTH.load(deps.storage).unwrap();
    let health_Factor =
        calculate_health_factor(collateral_value, token_minted, liquidity_threashold);
    println!("Current health_Factor is {}", health_Factor);
    if health_Factor.is_zero() {
        return Err(ContractError::HealthFactorLess {});
    }

    let token = STABLE.load(deps.storage)?;

    let msg = mint_stable(user.clone(), token_amount, token);

    Ok(Response::new().add_message(msg))
}

fn execute_redeem_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount_withdraw: Uint128,
) -> Result<Response, ContractError> {
    println!("withdraw req is {}", amount_withdraw);
    let deposit = COLLATERALDEPOSITED.load(deps.storage, info.sender.clone())?;
    let token_minted = TOKENSMINTED
        .load(deps.storage, info.sender.clone())
        .unwrap_or_default();

    let liquidity_threashold = LIQUIDATIONTH.load(deps.storage).unwrap();

    let remaining_collateral = calculate_collateral_usd(deposit - amount_withdraw);
    println!("remaining collateral  {}", remaining_collateral);

    let health_Factor =
        calculate_health_factor(remaining_collateral, token_minted, liquidity_threashold);

    println!("health factor redeem req is {}", health_Factor);

    if health_Factor.is_zero() {
        return Err(ContractError::HealthFactorLess {});
    }

    println!("withdraw req final  is {}", amount_withdraw);

    COLLATERALDEPOSITED.save(
        deps.storage,
        info.sender.clone(),
        &(deposit - amount_withdraw),
    )?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: "uom".to_string(),
            amount: amount_withdraw,
        }],
    };

    let cmsg: CosmosMsg = msg.into();

    let contract_balance = deps
        .querier
        .query_balance(env.contract.address.clone(), "uom")?;

    println!("Contract balance is {}", contract_balance.amount);

    Ok(Response::new().add_message(cmsg))
}

fn execute_redeem_collateral_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount_collateral: Uint128,
    amount_token: Uint128,
) -> Result<Response, ContractError> {
    let token_minted = TOKENSMINTED.load(deps.storage, info.sender.clone())?;
    let collateral_deposited = COLLATERALDEPOSITED.load(deps.storage, info.sender.clone())?;

    let new_token = token_minted - amount_token;
    let new_collateral = collateral_deposited - amount_collateral;

    let liquidity_threashold = LIQUIDATIONTH.load(deps.storage).unwrap();

    let health_factor = calculate_health_factor(
        calculate_collateral_usd(new_collateral),
        new_token,
        liquidity_threashold,
    );

    if health_factor.is_zero() {
        return Err(ContractError::HealthFactorLess {});
    }

    let msg = BankMsg::Send {
        to_address: info.sender.clone().to_string(),
        amount: vec![Coin {
            denom: "uom".to_string(),
            amount: amount_collateral,
        }],
    };

    let cmsg: CosmosMsg = msg.into();

    let token = STABLE.load(deps.storage)?;

    let burn_msg = burn_stable(info.sender, amount_token, token);

    Ok(Response::new().add_message(cmsg).add_message(burn_msg))
}

fn execute_liquidation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let collateral_deposited = COLLATERALDEPOSITED.load(deps.storage, user.clone())?;
    let token_minted = TOKENSMINTED.load(deps.storage, user.clone())?;

    let liquidity_threashold = LIQUIDATIONTH.load(deps.storage)?;

    let health_factor = calculate_health_factor(
        calculate_collateral_usd(collateral_deposited),
        token_minted,
        liquidity_threashold,
    );

    if health_factor > Uint128::zero() {
        return Err(ContractError::HealthFactorSafe {});
    }

    let new_amount = token_minted - amount;

    let collatera_value = calculate_usd_in_collateral(amount);

    let send_with_bonus =
        (collatera_value * Uint128::new(10)) / Uint128::new(100) + collatera_value;

    unimplemented!()
}

fn calculate_health_factor(
    collateral_value: Uint128,
    token_minted: Uint128,
    liquidity_threashold: Uint128,
) -> Uint128 {
    if token_minted.is_zero() {
        return Uint128::MAX;
    }
    let health_factor: Uint128 =
        (collateral_value * Uint128::new(100)) / (token_minted * liquidity_threashold);
    return health_factor;
}

fn amount_sent(sent_funds: Vec<Coin>) -> Uint128 {
    let amount = sent_funds
        .iter()
        .find(|coin| coin.denom == "uom")
        .map(|coin| coin.amount)
        .unwrap_or(Uint128::zero());
    return amount;
}

fn calculate_collateral_usd(amount: Uint128) -> Uint128 {
    return (amount * Uint128::new(2000000)) / Uint128::new(1000000);
}

fn calculate_usd_in_collateral(amount: Uint128) -> Uint128 {
    return (amount * Uint128::new(1000000)) / Uint128::new(2000000);
}

// fn deposit_collateral(user: Addr, amount: Uint128, deps: DepsMut) {}

fn mint_stable(recipient: Addr, mint_amount: Uint128, token: Addr) -> CosmosMsg {
    let mint_msg = cw20_base::msg::ExecuteMsg::Mint {
        recipient: recipient.into(),
        amount: Uint128::from(mint_amount),
    };

    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_json_binary(&mint_msg).unwrap(),
        funds: vec![],
    }
    .into();

    return msg;
}

fn burn_stable(user: Addr, burn_amount: Uint128, token: Addr) -> CosmosMsg {
    let burn_msg = cw20_base::msg::ExecuteMsg::BurnFrom {
        owner: user.into(),
        amount: burn_amount,
    };

    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_json_binary(&burn_msg).unwrap(),
        funds: vec![],
    }
    .into();

    return msg;
}

fn send_native(recipient: Addr, amount: Uint128) -> CosmosMsg {
    let send_msg = BankMsg::Send {
        to_address: recipient.clone().to_string(),
        amount: vec![Coin {
            denom: "uom".to_string(),
            amount: amount,
        }],
    };

    let msg: CosmosMsg = send_msg.into();
    return msg;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, Coin, Empty, Uint128};
    use cw20_base::contract;
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use super::*;

    fn stable_coin_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn cw20_stable() -> Box<dyn Contract<Empty>> {
        let cw20_contract =
            ContractWrapper::new(contract::execute, contract::instantiate, contract::query);
        Box::new(cw20_contract)
    }

    fn mint_native(app: &mut App, recipient: String, denom: String, amount: u128) {
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: recipient,
                amount: vec![coin(amount, denom)],
            },
        ))
        .unwrap();
    }

    fn deploy_stable_contract(cw20_id: u64, stable_id: u64, app: &mut App, sender: Addr) -> Addr {
        let contract_addrss = app
            .instantiate_contract(
                stable_id,
                sender.clone(),
                &InstantiateMsg {
                    stable_coin_id: cw20_id,
                    owner: Some(sender.to_string()),
                },
                &[],
                "StableEngine",
                None,
            )
            .unwrap();

        return contract_addrss;
    }

    fn deploy_cw20_contract(
        cw20_id: u64,
        stable_engine: Addr,
        app: &mut App,
        sender: Addr,
    ) -> Addr {
        let contract_addrss = app
            .instantiate_contract(
                cw20_id,
                sender.clone(),
                &cw20_base::msg::InstantiateMsg {
                    name: "auraUSD".into(),
                    symbol: "aUSD".into(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: stable_engine.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                "Cw20_token",
                None,
            )
            .unwrap();

        return contract_addrss;
    }

    fn deploy_all_contracts(mut app: App, user_addr: Addr, owner_addr: Addr) -> (App, Addr, Addr) {
        let user_addr = Addr::unchecked("sender");
        let owner_addr = Addr::unchecked("owner");

        let cw20_id = app.store_code(cw20_stable());

        let stable_engine = app.store_code(stable_coin_contract());
        mint_native(
            &mut app,
            user_addr.to_string(),
            "uom".to_string(),
            1_100_000u128,
        );

        let balances = app.wrap().query_all_balances(&user_addr).unwrap();

        for coin in balances {
            println!("{}: {}", coin.denom, coin.amount);
        }

        let stable_engine =
            deploy_stable_contract(cw20_id, stable_engine, &mut app, owner_addr.clone());

        let contract_addrss =
            deploy_cw20_contract(cw20_id, stable_engine.clone(), &mut app, owner_addr.clone());

        let execut_msg = ExecuteMsg::SetConfig {
            token: contract_addrss.clone(),
            threashold: Uint128::new(129),
        };

        let response = app
            .execute_contract(owner_addr.clone(), stable_engine.clone(), &execut_msg, &[])
            .unwrap();
        return (app, stable_engine, contract_addrss);
    }

    fn get_cw20_balance(owner_addr: Addr, app: App, contract_addrss: Addr) -> (Uint128, App) {
        let mut qur_msg = cw20_base::msg::QueryMsg::Balance {
            address: owner_addr.to_string(),
        };

        let mut qry_res: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(contract_addrss, &qur_msg)
            .unwrap();

        return (qry_res.balance, app);
    }

    #[test]
    fn deploy_contracts() {
        let oldapp = App::default();

        let user_addr = Addr::unchecked("sender");
        let owner_addr = Addr::unchecked("owner");

        let (mut app, stable_engine, contract_addrss) =
            deploy_all_contracts(oldapp, user_addr.clone(), owner_addr.clone());

        let (mut balance, mut app) =
            get_cw20_balance(user_addr.clone(), app, contract_addrss.clone());
        println!("balance in stable is {}", balance);

        let dep_msg = ExecuteMsg::DepositCollateralAndMint {
            token_amount: Uint128::from(1000u128),
        };

        let dep_response = app
            .execute_contract(
                user_addr.clone(),
                stable_engine.clone(),
                &dep_msg,
                &vec![coin(1300, "uom")],
            )
            .unwrap();

        let dep_msg2 = ExecuteMsg::DepositCollateralAndMint {
            token_amount: Uint128::from(1015u128),
        };

        let dep_response2 = app
            .execute_contract(user_addr.clone(), stable_engine.clone(), &dep_msg2, &[])
            .unwrap();

        (balance, app) = get_cw20_balance(user_addr.clone(), app, contract_addrss.clone());
        println!("balance in stable after is {}", balance);
    }

    #[test]
    fn test_deposit_and_mint() {
        let oldapp = App::default();

        let user_addr = Addr::unchecked("sender");
        let owner_addr = Addr::unchecked("owner");

        let (mut app, stable_engine, contract_addrss) =
            deploy_all_contracts(oldapp, user_addr.clone(), owner_addr.clone());

        let dep_msg = ExecuteMsg::DepositCollateralAndMint {
            token_amount: Uint128::from(1000u128),
        };

        let dep_response = app
            .execute_contract(
                user_addr.clone(),
                stable_engine.clone(),
                &dep_msg,
                &vec![coin(1300, "uom")],
            )
            .unwrap();

        let redeem_msg = ExecuteMsg::DepositCollateral {};

        let mut balances = app.wrap().query_all_balances(&user_addr).unwrap();

        for coin in balances {
            println!("beofre balance is {}: {}", coin.denom, coin.amount);
        }

        let redeem_response = app
            .execute_contract(
                user_addr.clone(),
                stable_engine.clone(),
                &redeem_msg,
                &vec![coin(1300, "uom")],
            )
            .unwrap();

        balances = app.wrap().query_all_balances(&user_addr).unwrap();

        for coin in balances {
            println!("rn balance is {}: {}", coin.denom, coin.amount);
        }
    }

    #[test]
    fn test_deposit_and_redeem() {
        let oldapp = App::default();

        let user_addr = Addr::unchecked("sender");
        let owner_addr = Addr::unchecked("owner");

        let (mut app, stable_engine, contract_addrss) =
            deploy_all_contracts(oldapp, user_addr.clone(), owner_addr.clone());

        let dep_msg = ExecuteMsg::DepositCollateralAndMint {
            token_amount: Uint128::from(1000u128),
        };

        let dep_response = app
            .execute_contract(
                user_addr.clone(),
                stable_engine.clone(),
                &dep_msg,
                &vec![coin(1300, "uom")],
            )
            .unwrap();

        let redeem_msg = ExecuteMsg::RedeemCollateral {
            amount: Uint128::new(655),
        };

        let mut balances = app.wrap().query_all_balances(&user_addr).unwrap();

        for coin in balances {
            println!("beofre balance is {}: {}", coin.denom, coin.amount);
        }

        let redeem_response = app
            .execute_contract(user_addr.clone(), stable_engine.clone(), &redeem_msg, &[])
            .unwrap();

        balances = app.wrap().query_all_balances(&user_addr).unwrap();

        for coin in balances {
            println!("rn balance is {}: {}", coin.denom, coin.amount);
        }
    }

    #[test]
    fn test_deposit_and_burn() {
        let oldapp = App::default();

        let user_addr = Addr::unchecked("sender");
        let owner_addr = Addr::unchecked("owner");

        let (mut app, stable_engine, contract_addrss) =
            deploy_all_contracts(oldapp, user_addr.clone(), owner_addr.clone());

        let dep_msg = ExecuteMsg::DepositCollateralAndMint {
            token_amount: Uint128::from(1000u128),
        };

        let dep_response = app
            .execute_contract(
                user_addr.clone(),
                stable_engine.clone(),
                &dep_msg,
                &vec![coin(1300, "uom")],
            )
            .unwrap();

        let burn_msg = ExecuteMsg::RedeemCollateralAndBurn {
            amount_collateral: Uint128::new(1300),
            amount_token: Uint128::new(1000),
        };

        let mut balances = app.wrap().query_all_balances(&user_addr).unwrap();

        for coin in balances {
            println!("beofre balance is {}: {}", coin.denom, coin.amount);
        }
        let mut stable_bal;
        (stable_bal, app) = get_cw20_balance(user_addr.clone(), app, contract_addrss.clone());
        println!("beofre stable_bal is {}", stable_bal);

        let allow_msg = cw20_base::msg::ExecuteMsg::IncreaseAllowance {
            spender: stable_engine.clone().into(),
            amount: Uint128::new(1000000),
            expires: None,
        };

        let allow_response = app
            .execute_contract(user_addr.clone(), contract_addrss.clone(), &allow_msg, &[])
            .unwrap();

        let redeem_response = app
            .execute_contract(user_addr.clone(), stable_engine.clone(), &burn_msg, &[])
            .unwrap();

        balances = app.wrap().query_all_balances(&user_addr).unwrap();

        (stable_bal, app) = get_cw20_balance(user_addr.clone(), app, contract_addrss);
        println!("rn stable_bal is {}", stable_bal);

        for coin in balances {
            println!("rn balance is {}: {}", coin.denom, coin.amount);
        }
    }
}
//cargo test -- --nocapture
