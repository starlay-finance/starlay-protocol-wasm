// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{
    controller::Error as ControllerError,
    pool::Error as PoolError,
};
use openbrush::{
    contracts::traits::access_control::AccessControlError,
    traits::{
        AccountId,
        Balance,
    },
};

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type ManagerRef = dyn Manager;

/// Trait for managing a lending pool (Controller, all pools etc)
#[openbrush::trait_definition]
pub trait Manager {
    /// Get the controller
    #[ink(message)]
    fn controller(&self) -> AccountId;

    /// Set the controller
    #[ink(message)]
    fn set_controller(&mut self, address: AccountId) -> Result<()>;

    /// Sets a new price oracle for the controller
    #[ink(message)]
    fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;

    /// Sets a new flashloan gateway for the controller
    #[ink(message)]
    fn set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()>;

    /// Add the market to the markets mapping and set it as listed (call Controller)
    #[ink(message)]
    fn support_market(&mut self, pool: AccountId, underlying: AccountId) -> Result<()>;

    /// Add the market to the markets mapping and set it as listed with collateral_factor (call Controller)
    #[ink(message)]
    fn support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        underlying: AccountId,
        collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;

    /// Sets the collateralFactor for a market (call Controller)
    #[ink(message)]
    fn set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;

    /// Update the pause status of mint action in the pool (call Controller)
    #[ink(message)]
    fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;

    /// Update the pause status of borrow action in the pool (call Controller)
    #[ink(message)]
    fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;

    /// Sets the closeFactor used when liquidating borrows (call Controller)
    #[ink(message)]
    fn set_close_factor_mantissa(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()>;

    /// Sets liquidationIncentive (call Controller)
    #[ink(message)]
    fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()>;

    /// Set the given borrow caps for the given pool (call Controller)
    #[ink(message)]
    fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()>;

    /// accrues interest and sets a new reserve factor for the protocol using _set_reserve_factor_mantissa (call Pool)
    #[ink(message)]
    fn set_reserve_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()>;

    /// Accrues interest and reduces reserves by transferring to admin (call Pool)
    #[ink(message)]
    fn reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()>;

    /// A public function to sweep accidental token transfers to this contract. (call Pool)
    #[ink(message)]
    fn sweep_token(&mut self, pool: AccountId, asset: AccountId) -> Result<()>;
}

/// Custom error definitions for Manager
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    AccessControl(AccessControlError),
    Controller(ControllerError),
    Pool(PoolError),
}

impl From<AccessControlError> for Error {
    fn from(error: AccessControlError) -> Self {
        Error::AccessControl(error)
    }
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

pub type Result<T> = core::result::Result<T, Error>;
