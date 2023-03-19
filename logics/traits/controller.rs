use ink::prelude::vec::Vec;
use openbrush::traits::{
    AccountId,
    Balance,
};

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type ControllerRef = dyn Controller;

#[openbrush::trait_definition]
pub trait Controller {
    #[ink(message)]
    fn mint_allowed(&self, pool: AccountId, minter: AccountId, mint_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn mint_verify(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn redeem_verify(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        redeem_tokens: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn borrow_verify(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn repay_borrow_allowed(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn repay_borrow_verify(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        borrower_index: u128,
    ) -> Result<()>;

    #[ink(message)]
    fn liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;

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

    #[ink(message)]
    fn seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn seize_verify(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn transfer_allowed(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn transfer_verify(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()>;

    #[ink(message)]
    fn liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        exchange_rate_mantissa: WrappedU256,
        repay_amount: Balance,
    ) -> Result<Balance>;

    // admin functions
    #[ink(message)]
    fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;

    #[ink(message)]
    fn support_market(&mut self, pool: AccountId) -> Result<()>;

    #[ink(message)]
    fn set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;

    #[ink(message)]
    fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;

    #[ink(message)]
    fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;

    #[ink(message)]
    fn set_close_factor_mantissa(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()>;

    #[ink(message)]
    fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()>;

    #[ink(message)]
    fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()>;

    // view function
    #[ink(message)]
    fn markets(&self) -> Vec<AccountId>;
    #[ink(message)]
    fn collateral_factor_mantissa(&self, pool: AccountId) -> Option<WrappedU256>;
    #[ink(message)]
    fn mint_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    #[ink(message)]
    fn borrow_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    #[ink(message)]
    fn oracle(&self) -> AccountId;
    #[ink(message)]
    fn close_factor_mantissa(&self) -> WrappedU256;
    #[ink(message)]
    fn liquidation_incentive_mantissa(&self) -> WrappedU256;
    #[ink(message)]
    fn borrow_cap(&self, pool: AccountId) -> Option<Balance>;
    #[ink(message)]
    fn manager(&self) -> AccountId;
    #[ink(message)]
    fn is_listed(&self, pool: AccountId) -> bool;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    MintIsPaused,
    BorrowIsPaused,
    MarketNotListed,
    ControllerMismatch,
    PriceError,
    TooMuchRepay,
    BorrowCapReached,
    CallerIsNotManager,
}

pub type Result<T> = core::result::Result<T, Error>;
