// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ink::prelude::vec::Vec;
use openbrush::traits::{
    AccountId,
    Balance,
};
use primitive_types::U256;
use scale::{
    Decode,
    Encode,
};

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type ControllerRef = dyn Controller;

/// Trait defines the interface for the controller of a lending protocol.
/// It contains a set of functions that are responsible for validating and calculating various actions related to lending, such as minting, borrowing, and liquidation.
#[openbrush::trait_definition]
pub trait Controller {
    /// Checks if the account should be allowed to mint tokens in the given market
    #[ink(message)]
    fn mint_allowed(&self, pool: AccountId, minter: AccountId, mint_amount: Balance) -> Result<()>;

    /// Validates mint and reverts on rejection. May emit logs.
    #[ink(message)]
    fn mint_verify(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    ) -> Result<()>;

    /// Checks if the account should be allowed to redeem tokens in the given market
    #[ink(message)]
    fn redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        pool_attribute: Option<PoolAttributesForWithdrawValidation>,
    ) -> Result<()>;

    /// Validates redeem and reverts on rejection. May emit logs.
    #[ink(message)]
    fn redeem_verify(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
    ) -> Result<()>;

    /// Checks if the account should be allowed to borrow the underlying asset of the given market
    #[ink(message)]
    fn borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()>;

    /// Validates borrow and reverts on rejection. May emit logs.
    #[ink(message)]
    fn borrow_verify(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()>;

    /// Checks if the account should be allowed to repay a borrow in the given market
    #[ink(message)]
    fn repay_borrow_allowed(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;

    /// Validates repayBorrow and reverts on rejection. May emit logs.
    #[ink(message)]
    fn repay_borrow_verify(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        borrower_index: u128,
    ) -> Result<()>;

    /// Checks if the liquidation should be allowed to occur
    #[ink(message)]
    fn liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()>;

    /// Validates liquidateBorrow and reverts on rejection. May emit logs.
    #[ink(message)]
    fn liquidate_borrow_verify(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        seize_tokens: Balance,
    ) -> Result<()>;

    /// Checks if the seizing of assets should be allowed to occur
    #[ink(message)]
    fn seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;

    /// Validates seize and reverts on rejection. May emit logs.
    #[ink(message)]
    fn seize_verify(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;

    /// Checks if the account should be allowed to transfer tokens in the given market
    #[ink(message)]
    fn transfer_allowed(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
        pool_attribute: Option<PoolAttributesForWithdrawValidation>,
    ) -> Result<()>;

    /// Validates transfer and reverts on rejection. May emit logs.
    #[ink(message)]
    fn transfer_verify(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()>;

    /// Checks if the account should be allowed to transfer tokens in the given market
    #[ink(message)]
    fn liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        exchange_rate_mantissa: WrappedU256,
        repay_amount: Balance,
        pool_borrowed_attributes: Option<PoolAttributesForSeizeCalculation>,
        pool_collateral_attributes: Option<PoolAttributesForSeizeCalculation>,
    ) -> Result<Balance>;

    // admin functions

    /// Sets a new price oracle for the controller
    #[ink(message)]
    fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;

    /// Add the market to the markets mapping and set it as listed
    #[ink(message)]
    fn support_market(&mut self, pool: AccountId, underlying: AccountId) -> Result<()>;

    #[ink(message)]
    fn set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()>;

    /// Add the market to the markets mapping and set it as listed with collateral_factor
    #[ink(message)]
    fn support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        underlying: AccountId,
        collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;

    /// Sets the collateralFactor for a market
    #[ink(message)]
    fn set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;

    /// Update the pause status of mint action in the pool
    #[ink(message)]
    fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;

    /// Update the pause status of borrow action in the pool
    #[ink(message)]
    fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;

    /// Update the pause status of seize action in the pool
    #[ink(message)]
    fn set_seize_guardian_paused(&mut self, paused: bool) -> Result<()>;

    /// Update the transfer status of seize action in the pool
    #[ink(message)]
    fn set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()>;

    /// Sets the closeFactor used when liquidating borrows
    #[ink(message)]
    fn set_close_factor_mantissa(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()>;

    /// Sets liquidationIncentive
    #[ink(message)]
    fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()>;

    /// Set the given borrow caps for the given pool.
    /// Borrowing that brings total borrows to or above borrow cap will revert.
    #[ink(message)]
    fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()>;

    // view function
    /// Returns the list of all markets that are currently supported
    #[ink(message)]
    fn markets(&self) -> Vec<AccountId>;

    #[ink(message)]
    fn flashloan_gateway(&self) -> Option<AccountId>;

    /// Returns the market based on underlying
    #[ink(message)]
    fn market_of_underlying(&self, underlying: AccountId) -> Option<AccountId>;

    /// Returns the collateral factor for a given pool
    #[ink(message)]
    fn collateral_factor_mantissa(&self, pool: AccountId) -> Option<WrappedU256>;

    /// Returns the current mint pause status for a given pool
    #[ink(message)]
    fn mint_guardian_paused(&self, pool: AccountId) -> Option<bool>;

    /// Returns the current borrow pause status for a given pool
    #[ink(message)]
    fn borrow_guardian_paused(&self, pool: AccountId) -> Option<bool>;

    /// Returns the current seize pause status
    #[ink(message)]
    fn seize_guardian_paused(&self) -> bool;

    /// Returns the current transfer pause status
    #[ink(message)]
    fn transfer_guardian_paused(&self) -> bool;

    /// Returns the price oracle account id
    #[ink(message)]
    fn oracle(&self) -> Option<AccountId>;

    /// Returns the close factor
    #[ink(message)]
    fn close_factor_mantissa(&self) -> WrappedU256;

    /// Returns the liquidation incentive
    #[ink(message)]
    fn liquidation_incentive_mantissa(&self) -> WrappedU256;

    /// Returns the borrow cap for a given pool
    #[ink(message)]
    fn borrow_cap(&self, pool: AccountId) -> Option<Balance>;

    /// Returns the account id of the manager account
    #[ink(message)]
    fn manager(&self) -> Option<AccountId>;

    /// Returns whether a given pool is currently listed
    #[ink(message)]
    fn is_listed(&self, pool: AccountId) -> bool;

    /// Returns a list of assets associated with a given account
    #[ink(message)]
    fn account_assets(&self, account: AccountId) -> Vec<AccountId>;

    /// Returns User account data
    #[ink(message)]
    fn calculate_user_account_data(
        &self,
        account: AccountId,
        pool_attributes: Option<PoolAttributesForWithdrawValidation>,
    ) -> Result<AccountData>;

    /// Check if withdraw is valid.
    #[ink(message)]
    fn balance_decrease_allowed(
        &self,
        pool_attributes: PoolAttributesForWithdrawValidation,
        account: AccountId,
        amount: Balance,
    ) -> Result<()>;
    /// Determine the current account liquidity with respect to collateral requirements
    #[ink(message)]
    fn get_account_liquidity(&self, account: AccountId) -> Result<(U256, U256)>;

    /// Determine what the account liquidity would be if the given amounts were redeemed/borrowed
    #[ink(message)]
    fn get_hypothetical_account_liquidity(
        &self,
        account: AccountId,
        token: AccountId,
        redeem_tokens: Balance,
        borrow_amount: Balance,
    ) -> Result<(U256, U256)>;
}

/// Structure for holding information about the Pool
///
/// NOTE: Used to prevent cross contract calls to the caller pool
#[derive(Clone, Decode, Encode, Default)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct PoolAttributes {
    pub underlying: Option<AccountId>,
    pub decimals: u8,
    pub account_balance: Balance,
    pub account_borrow_balance: Balance,
    pub exchange_rate: U256,
    pub total_borrows: Balance,
}

/// Structure for having information for Seize about the Pool
///
/// NOTE: Used to prevent cross contract calls to the caller pool
#[derive(Clone, Decode, Encode, Default)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct PoolAttributesForSeizeCalculation {
    pub underlying: Option<AccountId>,
    pub decimals: u8,
}

/// Structure for having information for Withdraw's validations about the Pool
///
/// NOTE: Used to prevent cross contract calls to the caller pool
#[derive(Clone, Decode, Encode, Default)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct PoolAttributesForWithdrawValidation {
    pub pool: Option<AccountId>,
    pub underlying: Option<AccountId>,
    pub decimals: u8,
    pub liquidation_threshold: u128,
    pub account_balance: Balance,
    pub account_borrow_balance: Balance,
    pub exchange_rate: U256,
    pub total_borrows: Balance,
}

/// Structure to hold status information of a user
///
/// Used to retrieve the status of all users in the Protocol pool and to make the calculated results available for use and reference.
#[derive(Clone, Decode, Encode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct AccountData {
    pub total_collateral_in_base_currency: U256,
    pub total_debt_in_base_currency: U256,
    pub avg_ltv: U256,
    pub avg_liquidation_threshold: U256,
    pub health_factor: U256,
}

/// Custom error definitions for Controller
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    MintIsPaused,
    BorrowIsPaused,
    SeizeIsPaused,
    TransferIsPaused,
    MarketNotListed,
    MarketAlreadyListed,
    ControllerMismatch,
    PriceError,
    TooMuchRepay,
    BorrowCapReached,
    InsufficientLiquidity,
    InsufficientShortfall,
    CallerIsNotManager,
    InvalidCollateralFactor,
    UnderlyingIsNotSet,
    PoolIsNotSet,
    ManagerIsNotSet,
    OracleIsNotSet,
    BalanceDecreaseNotAllowed,
}

pub type Result<T> = core::result::Result<T, Error>;
