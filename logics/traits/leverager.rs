// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use openbrush::{
    contracts::psp22::PSP22Error,
    traits::{
        AccountId,
        Balance,
    },
};
use primitive_types::U256;
use scale::{
    Decode,
    Encode,
};

use super::{
    controller::Error as ControllerError,
    pool::Error as PoolError,
};

#[openbrush::wrapper]
pub type LeveragerRef = dyn Leverager;

/// Trait defines the interface for the Leverager
#[openbrush::trait_definition]
pub trait Leverager {
    /// Get Controller AccountId
    #[ink(message)]
    fn controller(&self) -> Option<AccountId>;

    /// Get Price Oracle AccountId
    #[ink(message)]
    fn price_oracle(&self) -> Option<AccountId>;

    /// Get Weth AccountId
    #[ink(message)]
    fn weth_address(&self) -> Option<AccountId>;

    /// Get Manager AccountId
    #[ink(message)]
    fn manager(&self) -> Option<AccountId>;

    /// Get Borrowable information of an account
    #[ink(message)]
    fn get_available_borrows(&self, account: AccountId) -> Option<AvailableBorrows>;

    /// Get account health factor after withdraw
    #[ink(message)]
    fn get_health_factor(
        &self,
        account: AccountId,
        asset: AccountId,
        withdraw_amount: Balance,
    ) -> U256;

    /// Get withdrawable information for an account
    #[ink(message)]
    fn withdrawable(&self, account: AccountId, asset: AccountId) -> Option<Withdrawable>;

    /// Get withdrawable amount for an account asset
    #[ink(message)]
    fn withdrawable_amount(&self, account: AccountId, asset: AccountId) -> U256;

    /// Get Loan to value of the asset.
    #[ink(message)]
    fn loan_to_value(&self, asset: AccountId) -> u128;

    /// Get Liquidation threshold of the asset.
    #[ink(message)]
    fn liquidation_threshold(&self, asset: AccountId) -> u128;

    #[ink(message)]
    fn initialize(
        &mut self,
        controller: Option<AccountId>,
        price_oracle: Option<AccountId>,
        weth: Option<AccountId>,
    ) -> Result<()>;

    /// Loop the depositing and borrowing assets
    #[ink(message)]
    fn loop_asset(
        &mut self,
        asset: AccountId,
        amount: Balance,
        borrow_ratio: u128,
        loop_count: u128,
    ) -> Result<()>;

    /// Loop the depositing and borrowing eth
    #[ink(message, payable)]
    fn loop_eth(&mut self, borrow_ratio: u128, loop_count: u128) -> Result<()>;

    /// Loop the withdrawing and repaying asset
    #[ink(message)]
    fn close(&mut self, asset: AccountId) -> Result<()>;
}

#[derive(Clone, Decode, Encode, Default)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct AvailableBorrows {
    pub total_collateral_in_base_currency: U256,
    pub available_borrow_in_base_currency: U256,
    pub price_eth: Balance,
    pub health_factor: U256,
    pub ltv: U256,
}

#[derive(Clone, Decode, Encode, Default)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Withdrawable {
    pub total_collateral_in_base_currency: U256,
    pub total_debt_in_base_currency: U256,
    pub current_liquidation_threshold: U256,
    pub afford_in_base_currency: U256,
    pub withdrawable_collateral_in_base_currency: U256,
    pub withdrawable_collateral: U256,
    pub withdraw_amount: U256,
}

/// Custom error definitions for Controller
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    ManagerIsNotSet,
    CallerIsNotManager,
    InappropriateBorrowRate,
    InappropriateLoopCount,
    ControllerIsNotSet,
    MarketNotListed,
    WETHIsNotSet,
    Controller(ControllerError),
    Pool(PoolError),
    PSP22(PSP22Error),
}

impl From<ControllerError> for Error {
    fn from(error: ControllerError) -> Self {
        Error::Controller(error)
    }
}

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Error::Pool(error)
    }
}

impl From<PSP22Error> for Error {
    fn from(error: PSP22Error) -> Self {
        Error::PSP22(error)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
