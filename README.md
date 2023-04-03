# starlay-protocol-wasm

This repository is a Lending Protocol Template for WASM contracts using [ink!](https://use.ink/).

## Contracts

We detail a few of the contracts in this repository.

### Pool

The Pool contract is the core of the lending protocol. It is a contract that manages the assets of the user and the assets of the protocol.
The pool contanis the core logic of the pool itself and public interfaces for P2P22[https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md] tokens respectively. Each pool is assigned an interest rate and risk model(see DefaultInterestRateModel and Controller sections). The pool is also responsible for the transfer of assets between the user and the protocol. It allows accounts to deposit, borrow and repay assets.

### Controller

The Controller contract is a contract that manages the risk of the protocol. It is responsible for the risk model of the protocol and the risk model of each pool. It is also responsible for the management of the borrow_cap of each pool. The controller is also responsible for the management of the paused state of the protocol.

### DefaultInterestRateModel

The DefaultInterestRateModel contract is a contract that manages the interest rate of the protocol. The interest rate model is based on the Compound V2 interest rate model.

### Manager

The Manager contract is a contract that manages the protocol. It is responsible for the management of configurations of the controller and the pools.

### PriceOracle

The PriceOracle contract is a contract that manages the price of the assets. It is responsible for the management of the price of each asset.

## Instllation

To run starlay-protocol-wasm, pull the repository from GitHub and install the dependencies.

```bash
git clone https://github.com/starlay-finance/starlay-protocol-wasm.git
cd starlay-protocol-wasm
cargo build
```

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [cargo-contract](https://github.com/paritytech/cargo-contract)
- [swanky-cli](https://github.com/AstarNetwork/swanky-cli)

## Testing

### Unit Tests

To run the unit tests, run the following command:

```bash
cargo test
```

### End-to-End Tests

Before running the tests, you need to run the local node and deploy the

```bash
swanky node start
```

To run the end-to-end tests, run the following command:

```bash
yarn test
```

## Deployment

### to Local Node

To deploy the contracts to a local node, run the following command:

```bash
yarn deploy:local
```

### to Astar Testnet(Shibuya)

To deploy the contracts to the Astar Testnet(Shibuya), run the following command:

```bash
yarn deploy:shibuya
```
