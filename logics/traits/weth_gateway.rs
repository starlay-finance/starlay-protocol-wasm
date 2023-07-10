// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use openbrush::{
    contracts::{
        ownable::*,
        psp22::extensions::metadata::*,
    },
    modifiers,
    traits::{
        AccountId,
        Balance,
    },
};

pub use super::pool::Error as PoolError;

#[openbrush::wrapper]
pub type WETHGatewayRef = dyn WETHGateway + Ownable;

#[openbrush::trait_definition]
pub trait WETHGateway: Ownable {
    #[ink(message)]
    #[modifiers(only_owner)]
    fn authorize_pool(&mut self, pool: AccountId) -> Result<()>;

    /// Deposits WETH into the reserve, using native ETH. A corresponding amount of the overlying asset (lTokens) is minted.
    #[ink(message, payable)]
    fn deposit_eth(&mut self, pool: AccountId) -> Result<()>;

    /// Withdraws the WETH _reserves of caller.
    #[ink(message)]
    fn withdraw_eth(&mut self, pool: AccountId, amount: Balance) -> Result<()>;

    /// Repays a borrow on the WETH reserve, for the specified amount (or for the whole amount, if Balance::MAX is specified).
    #[ink(message, payable)]
    fn repay_eth(&mut self, pool: AccountId, amount: Balance) -> Result<()>;

    /// Borrow WETH, unwraps to ETH and send both the ETH and DebtTokens to caller, via `approveDelegation` and onBehalf argument in `pool.borrow`.
    #[ink(message)]
    fn borrow_eth(&mut self, pool: AccountId, amount: Balance) -> Result<()>;

    /// Transfer PSP22 from the utility contract, for PSP22 recovery in case of stuck tokens due direct transfers to the contract address.
    #[ink(message)]
    #[modifiers(only_owner)]
    fn emergency_token_transfer(
        &mut self,
        token: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> Result<()>;

    /// Transfer native Token from the utility contract, for native Token recovery in case of stuck Token
    #[ink(message)]
    #[modifiers(only_owner)]
    fn emergency_ether_transfer(&mut self, to: AccountId, amount: Balance) -> Result<()>;

    /// Get WETH address used by WETHGateway
    #[ink(message)]
    fn get_weth_address(&self) -> AccountId;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    SafeETHTransferFailed,
    InsufficientPayback,
    Pool(PoolError),
    PSP22(PSP22Error),
}

impl From<PSP22Error> for Error {
    fn from(error: PSP22Error) -> Self {
        Error::PSP22(error)
    }
}

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Error::Pool(error)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
