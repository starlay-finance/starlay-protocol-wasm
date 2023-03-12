pub use crate::traits::{
    controller::ControllerRef,
    pool::*,
};
use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp22::{
        self,
        Internal as PSP22Internal,
        PSP22Ref,
    },
    traits::{
        AccountId,
        Balance,
        Storage,
        ZERO_ADDRESS,
    },
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub underlying: AccountId,
    pub controller: AccountId,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            underlying: ZERO_ADDRESS.into(),
            controller: ZERO_ADDRESS.into(),
        }
    }
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

    fn _underlying(&self) -> AccountId;
    fn _controller(&self) -> AccountId;
}

impl<T: Storage<Data> + Storage<psp22::Data>> Pool for T {
    default fn mint(&mut self, mint_amount: Balance) -> Result<()> {
        self._accrue_interest();
        self._mint(Self::env().caller(), mint_amount)?;
        // TODO: event emission
        Ok(())
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

    fn underlying(&self) -> AccountId {
        self._underlying()
    }

    fn controller(&self) -> AccountId {
        self._controller()
    }
}

impl<T: Storage<Data> + Storage<psp22::Data>> Internal for T {
    default fn _accrue_interest(&mut self) {
        // todo!()
    }
    default fn _mint(&mut self, minter: AccountId, mint_amount: Balance) -> Result<()> {
        let contract_addr = Self::env().account_id();
        ControllerRef::mint_allowed(&self._controller(), contract_addr, minter, mint_amount)
            .unwrap();
        // TODO: assertion check - compare current block number with accrual block number

        // TODO: calculate exchange rate & mint amount
        let actual_mint_amount = mint_amount;
        // PSP22Ref::transfer_from(
        //     &self._underlying(),
        //     minter,
        //     minter,
        //     mint_amount,
        //     Vec::<u8>::new(),
        // )
        // .unwrap(); // TODO
        self._mint_to(minter, actual_mint_amount).unwrap();

        Ok(())
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

    fn _underlying(&self) -> AccountId {
        self.data::<Data>().underlying
    }

    fn _controller(&self) -> AccountId {
        self.data::<Data>().controller
    }
}
