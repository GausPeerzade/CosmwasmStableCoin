# CW-Stablecoin Smart Contract

This repository contains a CosmWasm smart contract designed to create and manage a stablecoin system backed by collateral.

## Overview

The contract allows users to:
- Deposit collateral
- Mint stablecoins
- Redeem collateral
- Burn stablecoins
- Liquidate positions

## Maintaining the $1 Peg

The protocol maintains the $1 peg of the stablecoin through the following mechanisms:

1. **Collateral Backing**:
   - Users deposit collateral (e.g., a native token) which is valued based on an external price oracle.
   - The value of the collateral is used to ensure that each stablecoin minted is backed by sufficient collateral.

2. **Health Factor**:
   - The health factor is calculated as the ratio of the collateral value to the debt (stablecoins minted).
   - A minimum health factor ensures that the value of the collateral always exceeds the value of the minted stablecoins.

3. **Liquidation Mechanism**:
   - If a user's health factor falls below a predefined threshold, their position can be liquidated.
   - Liquidation involves burning the user's stablecoins and transferring the equivalent value of collateral to the liquidator, ensuring that the stablecoin remains fully backed.

Reference: [calculate_health_factor function](src/contract.rs#L200)

## Features

### Initialization

The contract is initialized with the following parameters:
- Owner address
- Oracle address
- Denomination of the collateral
- Minimum and liquidity thresholds

Reference: [instantiate function](src/contract.rs#L23)

### Execute Functions

The contract supports several execute functions to manage the stablecoin system:

- `SetToken`: Sets the stablecoin token address.
- `DepositCollateral`: Allows users to deposit collateral.
- `DepositCollateralAndMint`: Allows users to deposit collateral and mint stablecoins.
- `RedeemCollateral`: Allows users to redeem their collateral.
- `RedeemCollateralAndBurn`: Allows users to redeem collateral and burn stablecoins.
- `Liquidate`: Allows liquidation of a user's position if their health factor is below the safe threshold.
- `Swap`: Allows users to swap stablecoins for collateral at the current oracle price.

Reference: [execute function](src/contract.rs#L53)

### Helper Functions

- `calculate_health_factor`: Calculates the health factor (collateral value relative to debt).
- `amount_sent`: Calculates the amount of a specific denomination sent by the user.
- `calculate_collateral_usd`: Converts the amount of collateral to its USD value using the oracle price.
- `oracle_price`: Fetches the price of the collateral from the oracle.

Reference: [helper functions](src/contract.rs#L200)

### Minting and Burning Stablecoins

- `mint_stable`: Mints new stablecoins and sends them to the user.
- `burn_stable`: Burns a specified amount of stablecoins from the user's balance.

Reference: [minting and burning functions](src/contract.rs#L265)

### Query Functions

- `Config`: Retrieves the current configuration of the contract.
- `Info`: Retrieves information about a user's collateral, total debt, and health factor.

Reference: [query function](src/contract.rs#L300)

## State Management

The contract tracks the following states:
- Configuration (`CONFIG`)
- Collateral deposited by users (`COLLATERALDEPOSITED`)
- Tokens minted (`TOKENSMINTED`)
- Stablecoin token address (`STABLE`)

Reference: [state management](src/state.rs)

## Error Handling

Custom errors are defined to handle various failure scenarios such as unauthorized actions or insufficient health factors.

Reference: [error definitions](src/error.rs)

## Contract Version

The contract version is set using the `set_contract_version` function from the `cw2` crate.

Reference: [contract version](src/contract.rs#L17)

## Building and Testing

To build and test the contract, you can use the following commands:

```sh
cargo wasm
cargo test
