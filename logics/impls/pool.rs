pub use crate::traits::pool::*;
use openbrush::traits::{
    AccountId,
    Balance,
    Storage,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    // TODO
}

pub trait Internal {
    fn _accrue_interest(&mut self);
    fn _mint(&mut self, minter: AccountId, mint_amount: Balance) -> Result<()>;
    fn _redeem(
        &mut self,
        redeemer: AccountId,
        redeem_tokens: Balance,
        redeem_amount: Balance,
    ) -> Result<()>;
    fn _borrow(&mut self, borrower: AccountId, borrow_amount: Balance) -> Result<()>;
    fn _repay_borrow(
        &mut self,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;
    fn _liquidate_borrow(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()>;
    fn _seize(
        &mut self,
        seizer_token: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: AccountId,
    ) -> AccountId;
}

impl<T: Storage<Data>> Pool for T {
    default fn mint(&mut self, mint_amount: Balance) -> Result<()> {
        self._accrue_interest();
        self._mint(Self::env().caller(), mint_amount)
    }

    default fn redeem(&mut self, redeem_tokens: Balance) -> Result<()> {
        self._accrue_interest();
        self._redeem(Self::env().caller(), redeem_tokens, 0)
    }

    default fn redeem_underlying(&mut self, redeem_amount: Balance) -> Result<()> {
        self._accrue_interest();
        self._redeem(Self::env().caller(), 0, redeem_amount)
    }

    default fn borrow(&mut self, borrow_amount: Balance) -> Result<()> {
        self._accrue_interest();
        self._borrow(Self::env().caller(), borrow_amount)
    }

    default fn repay_borrow(&mut self, repay_amount: Balance) -> Result<()> {
        self._accrue_interest();
        self._repay_borrow(Self::env().caller(), Self::env().caller(), repay_amount)
    }

    default fn repay_borrow_behalf(
        &mut self,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        self._accrue_interest();
        self._repay_borrow(Self::env().caller(), borrower, repay_amount)
    }

    default fn liquidate_borrow(
        &mut self,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()> {
        self._accrue_interest();
        self._liquidate_borrow(Self::env().caller(), borrower, repay_amount, collateral)
    }

    default fn seize(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: AccountId,
    ) -> AccountId {
        self._accrue_interest();
        self._seize(Self::env().caller(), liquidator, borrower, seize_tokens)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _accrue_interest(&mut self) {
        todo!()
    }
    default fn _mint(&mut self, _minter: AccountId, _mint_amount: Balance) -> Result<()> {
        todo!()
    }
    default fn _redeem(
        &mut self,
        _redeemer: AccountId,
        _redeem_tokens: Balance,
        _redeem_amount: Balance,
    ) -> Result<()> {
        todo!()
    }
    default fn _borrow(&mut self, _borrower: AccountId, _borrow_amount: Balance) -> Result<()> {
        todo!()
    }
    default fn _repay_borrow(
        &mut self,
        _payer: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
    ) -> Result<()> {
        todo!()
    }
    default fn _liquidate_borrow(
        &mut self,
        _liquidator: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
        _collateral: AccountId,
    ) -> Result<()> {
        todo!()
    }
    default fn _seize(
        &mut self,
        _seizer_token: AccountId,
        _liquidator: AccountId,
        _borrower: AccountId,
        _seize_tokens: AccountId,
    ) -> AccountId {
        todo!()
    }
}
