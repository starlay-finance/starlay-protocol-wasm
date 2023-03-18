use ink::LangError;
use openbrush::{
    contracts::{
        psp22::PSP22Error,
        traits::psp22::*,
    },
    traits::{
        AccountId,
        Balance,
        Timestamp,
    },
};

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type PoolRef = dyn Pool + PSP22;

#[openbrush::trait_definition]
pub trait Pool: PSP22 {
    #[ink(message)]
    fn mint(&mut self, mint_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn redeem(&mut self, redeem_tokens: Balance) -> Result<()>;

    #[ink(message)]
    fn redeem_underlying(&mut self, redeem_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn borrow(&mut self, borrow_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn repay_borrow(&mut self, repay_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn repay_borrow_behalf(&mut self, borrower: AccountId, repay_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn liquidate_borrow(
        &mut self,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()>;

    #[ink(message)]
    fn seize(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;
    #[ink(message)]
    fn reduce_reserves(&mut self, amount: Balance) -> Result<()>;

    #[ink(message)]
    fn underlying(&self) -> AccountId;
    #[ink(message)]
    fn controller(&self) -> AccountId;
    #[ink(message)]
    fn manager(&self) -> AccountId;
    #[ink(message)]
    fn get_cash_prior(&self) -> Balance;
    #[ink(message)]
    fn total_borrows(&self) -> Balance;
    #[ink(message)]
    fn borrow_balance_stored(&self, account: AccountId) -> Balance;
    #[ink(message)]
    fn get_accrual_block_timestamp(&self) -> Timestamp;
    #[ink(message)]
    fn exchage_rate_stored(&self) -> WrappedU256;
    #[ink(message)]
    fn exchange_rate_current(&mut self) -> Result<WrappedU256>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    NotImplemented,
    InvalidParameter,
    OnlyEitherRedeemTokensOrRedeemAmountIsZero,
    BorrowCashNotAvailable,
    RedeemTransferOutNotPossible,
    LiquidateLiquidatorIsBorrower,
    LiquidateCloseAmountIsZero,
    AccrualBlockNumberIsNotFresh,
    LiquidateSeizeLiquidatorIsBorrower,
    ReduceReservesCashNotAvailable,
    ReduceReservesCashValidation,
    BorrowRateIsAbsurdlyHigh,
    CallerIsNotManager,
    PSP22(PSP22Error),
    Lang(LangError),
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
