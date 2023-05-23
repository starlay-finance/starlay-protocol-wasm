[![ci](https://github.com/starlay-finance/starlay-protocol-wasm/actions/workflows/ci.yml/badge.svg)](https://github.com/starlay-finance/starlay-protocol-wasm/actions/workflows/ci.yml)

# starlay-protocol-wasm

This repository is a Lending Protocol Template for WASM contracts using [ink!](https://use.ink/).

## Contracts

We detail a few of the contracts in this repository.

### Pool

Pool is the core of the lending protocol. The contract manages the assets of the user and the assets of the protocol.

- The pool contains the core logic of the pool itself and public interfaces for P2P22[https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md] tokens respectively.
- Each pool is assigned an interest rate and risk model (see DefaultInterestRateModel and Controller sections).
- The pool is also responsible for the transfer of assets between the user and the protocol.
  - It allows accounts to deposit, borrow and repay assets.

### Controller

The Controller manages the risk of the protocol.
It is responsible for followings

- the risk model of the protocol and each pool
- the management of the borrow_cap of each pool
- the management of the paused state of the protocol

### DefaultInterestRateModel

The DefaultInterestRateModel contract manages the interest rate of the protocol.

The interest rate model is based on the Compound V2 interest rate model.

### Manager

The Manager manages the protocol configurations.

It is responsible for the management of configurations of the controller and the pools.

### PriceOracle

The PriceOracle contract manages the price of the assets.

It is responsible for the management of the price of each asset.

### Wrapped ETH Gateway

Wrapped ETH Gateway allows users to deposit, withdraw, borrow and repay using Native Token.
It interacts with Native Token pool and Wrapped Token.

### Flash Loan Gateway

A user can use liquidity in Starlay’s pools to use in another place in the same transaction, as long as the borrowed amount is returned before the end of the transaction.

## Architecture

Here, we will provide an explanation of the templates constructed in this repository.

- Based on the code of Compound on Ethereum.
- using the following as the core libraries for the contracts in this template.
  - [openbrush](https://github.com/727-Ventures/openbrush-contracts)
    - framework for ink! development (equivalent to OpenZeppelin in Ethereum)
  - [primitive-types](https://github.com/paritytech/parity-common/tree/master/primitive-types)
    - primitive types shared by Substrate and Parity Ethereum
      - U256 and others commonly used in Ethereum and its encoding/decoding

## Project Structure

```txt
(root)
|--- contracts: ... Smart contract definitions
|--- logics: ... Components that compose the smart contracts
| |- impls: ... State / logic implementations
| L- traits: ... Interfaces
|--- scripts: ... Utilities for offchain activities (deploy, e2e etc)
L--- tests: ... End-to-end tests
```

## Customize

The implementation is based on the interface of Compound V2.
It includes several customizations, and we will provide a brief overview of them.

### Functions

- Pool’s decimals is equal to the underlying
  - In Compound, the decimals of cToken are uniformly set to 8
  - Affects due to this change
    - the number of significant digits used when calculating liquidity is 18
      - This is because the minimum unit of the amount varies for each Pool
- balance_of
  - return the value converted to the quantity in underlying
- interest_rate_model

### Extensions

#### Liquidation Threshold

The liquidation threshold is the percentage at which a position is defined as undercollateralized.
The delta between the LTV and the Liquidation Threshold is a safety mechanism in place for borrowers.
For more detail, please look at [here](https://docs.starlay.finance/asset/risk-parameters#liquidation-threshold)

#### Switch assets not to be collateralized.

- A user can configure whether his/her asset to use as collateral or not
- If he/she configures an asset not to use as collateral, the asset is excluded from collateral amount calculation and liquidation target

### Others

- Events
  - We have implemented events that mainly focus on operations that use assets
    - such as mint, redeem, repay, and borrow
  - The interface for triggering events is in compliance with Compound standards, so users can add events as they like.
- Permission
  - We use Role Based Access Control implemented with OpenBrush's access_control
  - The defined/used roles are as follows:
    - DEFAULT_ADMIN_ROLE: management of the manager itself
    - CONTROLLER_ADMIN: management of the controller
    - TOKEN_ADMIN: management of the pool
    - BORROW_CAP_GUARDIAN: operator of the controller's borrow_cap
    - PAUSE_GUARDIAN: operator of the controller's paused state operation

## How to use

### Instllation

To run starlay-protocol-wasm, pull the repository from GitHub and install the dependencies.

```bash
git clone https://github.com/starlay-finance/starlay-protocol-wasm.git
cd starlay-protocol-wasm
cargo build
```

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [cargo-contract](https://github.com/paritytech/cargo-contract)
- [swanky-cli](https://github.com/AstarNetwork/swanky-cli)

### Testing

#### Unit Tests

To run the unit tests, run the following command:

```bash
cargo test
```

#### End-to-End Tests

Before running the tests, you need to run the local node and deploy the

```bash
swanky node start
```

To run the end-to-end tests, run the following command:

```bash
yarn test
```

### Deployment

#### to Local Node

To deploy the contracts to a local node, run the following command:

```bash
yarn deploy:local
```

#### to Astar Testnet(Shibuya)

To deploy the contracts to the Astar Testnet(Shibuya), run the following command:

```bash
yarn deploy:shibuya
```
