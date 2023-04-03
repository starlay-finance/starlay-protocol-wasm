# Description

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
