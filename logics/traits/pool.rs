// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ink::LangError;
use openbrush::{
    contracts::{
        psp22::PSP22Error,
        traits::psp22::{
            extensions::metadata::*,
            *,
        },
    },
    traits::{
        AccountId,
        Balance,
        Timestamp,
    },
};
use primitive_types::U256;

use super::{
    controller::Error as ControllerError,
    types::WrappedU256,
};

#[openbrush::wrapper]
pub type PoolRef = dyn Pool + PSP22 + PSP22Metadata;

/// Trait implemented by all pools
#[openbrush::trait_definition]
pub trait Pool: PSP22 + PSP22Metadata {
    /// Applies accrued interest to total borrows and reserves
    #[ink(message)]
    fn accrue_interest(&mut self) -> Result<()>;

    /// Sender supplies assets into the market and receives pool tokens in exchange
    #[ink(message)]
    fn mint(&mut self, mint_amount: Balance) -> Result<()>;

    /// Sender supplies assets into the market and receives pool tokens in exchange
    #[ink(message)]
    fn mint_to(&mut self, mint_account: AccountId, mint_amount: Balance) -> Result<()>;

    /// Sender redeems pool tokens in exchange for the underlying asset
    #[ink(message)]
    fn redeem(&mut self, redeem_tokens: Balance) -> Result<()>;

    /// Sender redeems pool tokens in exchange for a specified amount of underlying asset
    #[ink(message)]
    fn redeem_underlying(&mut self, redeem_amount: Balance) -> Result<()>;

    /// Sender redeems pool tokens in exchange for all amount of underlying asset
    #[ink(message)]
    fn redeem_all(&mut self) -> Result<()>;

    /// Sender borrows assets from the protocol to their own address
    #[ink(message)]
    fn borrow(&mut self, borrow_amount: Balance) -> Result<()>;

    /// borrows assets from the protocol to Borrower
    #[ink(message)]
    fn borrow_for(&mut self, borrower: AccountId, borrow_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn borrow_for_flashloan(&mut self, borrower: AccountId, borrow_amount: Balance) -> Result<()>;

    /// Sender repays their own borrow
    #[ink(message)]
    fn repay_borrow(&mut self, repay_amount: Balance) -> Result<()>;

    /// Sender repays all their own borrow
    #[ink(message)]
    fn repay_borrow_all(&mut self) -> Result<()>;

    /// Sender repays a borrow belonging to borrower
    #[ink(message)]
    fn repay_borrow_behalf(&mut self, borrower: AccountId, repay_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn borrows_scaled(&self) -> Balance;

    /// The sender liquidates the borrowers collateral.
    #[ink(message)]
    fn liquidate_borrow(
        &mut self,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()>;

    /// Transfers collateral tokens (this market) to the liquidator.
    #[ink(message)]
    fn seize(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;

    /// Transfer Underlying Token.
    #[ink(message)]
    fn transfer_underlying(&mut self, to: AccountId, amount: Balance) -> Result<()>;

    // admin functions
    /// Sets a new controller for the market
    #[ink(message)]
    fn set_controller(&mut self, new_controller: AccountId) -> Result<()>;

    /// accrues interest and sets a new reserve factor for the protocol using _set_reserve_factor_mantissa
    #[ink(message)]
    fn set_reserve_factor_mantissa(
        &mut self,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()>;

    /// accrues interest and updates the interest rate model using _set_interest_rate_model
    #[ink(message)]
    fn set_interest_rate_model(&mut self, new_interest_rate_model: AccountId) -> Result<()>;

    /// Set Liquidation Threshold
    #[ink(message)]
    fn set_liquidation_threshold(&mut self, new_liquidation_threshold: u128) -> Result<()>;

    /// The sender adds to reserves.
    #[ink(message)]
    fn add_reserves(&mut self, amount: Balance) -> Result<()>;

    /// Accrues interest and reduces reserves by transferring to admin
    #[ink(message)]
    fn reduce_reserves(&mut self, amount: Balance) -> Result<()>;

    /// A public function to sweep accidental token transfers to this contract. Tokens are sent to admin
    #[ink(message)]
    fn sweep_token(&mut self, asset: AccountId) -> Result<()>;

    /// Delegates borrowing power to a user on the specific debt token
    #[ink(message)]
    fn approve_delegate(&mut self, delegatee: AccountId, amount: Balance) -> Result<()>;

    /// Increase delegate allowance from owner
    #[ink(message)]
    fn increase_delegate_allowance(
        &mut self,
        owner: AccountId,
        delegatee: AccountId,
        amount: Balance,
    ) -> Result<()>;

    /// Decrease delegate allowance from owner
    #[ink(message)]
    fn decrease_delegate_allowance(
        &mut self,
        owner: AccountId,
        delegatee: AccountId,
        amount: Balance,
    ) -> Result<()>;

    /// Set whether user's asset to use as collateral or not
    #[ink(message)]
    fn set_use_reserve_as_collateral(&mut self, use_as_collateral: bool) -> Result<()>;

    // view functions
    #[ink(message)]
    fn underlying(&self) -> Option<AccountId>;
    #[ink(message)]
    fn controller(&self) -> Option<AccountId>;
    #[ink(message)]
    fn manager(&self) -> Option<AccountId>;
    #[ink(message)]
    fn get_cash_prior(&self) -> Balance;
    #[ink(message)]
    fn total_borrows(&self) -> Balance;
    #[ink(message)]
    fn total_reserves(&self) -> Balance;
    #[ink(message)]
    fn get_account_snapshot(&self, account: AccountId) -> (Balance, Balance, U256);
    #[ink(message)]
    fn borrow_balance_stored(&self, account: AccountId) -> Balance;
    #[ink(message)]
    fn borrow_balance_current(&mut self, account: AccountId) -> Result<Balance>;
    #[ink(message)]
    fn get_accrual_block_timestamp(&self) -> Timestamp;
    #[ink(message)]
    fn borrow_rate_per_msec(&self) -> WrappedU256;
    #[ink(message)]
    fn supply_rate_per_msec(&self) -> WrappedU256;
    #[ink(message)]
    fn exchange_rate_stored(&self) -> WrappedU256;
    #[ink(message)]
    fn exchange_rate_current(&mut self) -> Result<WrappedU256>;
    #[ink(message)]
    fn principal_balance_of(&self, account: AccountId) -> Balance;
    #[ink(message)]
    fn principal_total_supply(&self) -> Balance;
    #[ink(message)]
    fn initial_exchange_rate_mantissa(&self) -> WrappedU256;
    #[ink(message)]
    fn reserve_factor_mantissa(&self) -> WrappedU256;
    #[ink(message)]
    fn liquidation_threshold(&self) -> u128;
    /// Returns the delegation allowance of the user
    #[ink(message)]
    fn delegate_allowance(&self, owner: AccountId, delegatee: AccountId) -> Balance;
    /// Check if user is using reserve as collateral or not
    #[ink(message)]
    fn using_reserve_as_collateral(&self, user: AccountId) -> bool;
}

/// Custom error definitions for Pool
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    NotImplemented,
    InvalidParameter,
    BorrowCashNotAvailable,
    RedeemTransferOutNotPossible,
    LiquidateLiquidatorIsBorrower,
    LiquidateCloseAmountIsZero,
    AccrualBlockNumberIsNotFresh,
    LiquidateSeizeLiquidatorIsBorrower,
    ReduceReservesCashNotAvailable,
    ReduceReservesCashValidation,
    BorrowRateIsAbsurdlyHigh,
    SetReserveFactorBoundsCheck,
    CannotSweepUnderlyingToken,
    CallerIsNotManager,
    ZeroOwnerAddress,
    ZeroDelegateeAddress,
    InsufficientDelegateAllowance,
    DepositAlreadyInUse,
    CallerIsNotFlashloanGateway,
    ControllerIsNotSet,
    InterestRateModelIsNotSet,
    UnderlyingIsNotSet,
    ManagerIsNotSet,
    Controller(ControllerError),
    PSP22(PSP22Error),
    Lang(LangError),
}

impl From<ControllerError> for Error {
    fn from(error: ControllerError) -> Self {
        Error::Controller(error)
    }
}

impl From<PSP22Error> for Error {
    fn from(error: PSP22Error) -> Self {
        Error::PSP22(error)
    }
}

impl From<LangError> for Error {
    fn from(error: LangError) -> Self {
        Error::Lang(error)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
