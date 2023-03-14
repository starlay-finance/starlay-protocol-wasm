pub use crate::traits::{
    controller::ControllerRef,
    pool::*,
};
use ink::{
    prelude::vec::Vec,
    LangError,
};
use openbrush::{
    contracts::psp22::{
        self,
        Internal as PSP22Internal,
        PSP22Ref,
    },
    storage::Mapping,
    traits::{
        AccountId,
        Balance,
        Storage,
        ZERO_ADDRESS,
    },
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct BorrowSnapshot {
    principal: Balance,
    interest_index: u128,
}

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub underlying: AccountId,
    pub controller: AccountId,
    pub total_borrows: Balance,
    pub account_borrows: Mapping<AccountId, BorrowSnapshot>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            underlying: ZERO_ADDRESS.into(),
            controller: ZERO_ADDRESS.into(),
            total_borrows: Default::default(),
            account_borrows: Default::default(),
        }
    }
}

pub trait Internal {
    fn _accrue_interest(&mut self);
    fn _mint(&mut self, minter: AccountId, mint_amount: Balance) -> Result<()>;
    fn _redeem(
        &mut self,
        redeemer: AccountId,
        redeem_tokens_in: Balance,
        redeem_amount_in: Balance,
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
    fn _get_cash_prior(&self) -> Balance;
    fn _total_borrows(&self) -> Balance;
    fn _borrow_balance_stored(&self, account: AccountId) -> Balance;

    // event emission
    // fn _emit_accrue_interest_event(&self);
    fn _emit_mint_event(&self, minter: AccountId, mint_amount: Balance, mint_tokens: Balance);
    fn _emit_redeem_event(
        &self,
        redeemer: AccountId,
        redeem_amount: Balance,
        redeem_tokens: Balance,
    );
    fn _emit_borrow_event(
        &self,
        borrower: AccountId,
        borrow_amount: Balance,
        account_borrows: Balance,
        total_borrows: Balance,
    );
    fn _emit_repay_borrow_event(
        &self,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        account_borrows: Balance,
        total_borrows: Balance,
    );
    fn _emit_liquidate_borrow_event(
        &self,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        token_collateral: Balance,
        seize_tokens: Balance,
    );
}

impl<T: Storage<Data> + Storage<psp22::Data>> Pool for T {
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

    default fn underlying(&self) -> AccountId {
        self._underlying()
    }

    default fn controller(&self) -> AccountId {
        self._controller()
    }

    default fn get_cash_prior(&self) -> Balance {
        self._get_cash_prior()
    }

    default fn total_borrows(&self) -> Balance {
        self._total_borrows()
    }

    default fn borrow_balance_stored(&self, account: AccountId) -> Balance {
        self._borrow_balance_stored(account)
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
        PSP22Ref::transfer_from_builder(
            &self._underlying(),
            minter,
            contract_addr,
            actual_mint_amount,
            Vec::<u8>::new(),
        )
        .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))
        .try_invoke()
        .unwrap()
        .unwrap()
        .unwrap();
        self._mint_to(minter, mint_amount).unwrap();

        self._emit_mint_event(minter, actual_mint_amount, mint_amount);

        Ok(())
    }
    default fn _redeem(
        &mut self,
        redeemer: AccountId,
        redeem_tokens_in: Balance,
        redeem_amount_in: Balance,
    ) -> Result<()> {
        let exchange_rate = 1; // TODO: calculate exchange rate & redeem amount
        let (redeem_tokens, redeem_amount) = match (redeem_tokens_in, redeem_amount_in) {
            (tokens, _) if tokens > 0 => (tokens, tokens * exchange_rate),
            (_, amount) if amount > 0 => (amount / exchange_rate, amount),
            _ => return Err(Error::InvalidParameter),
        };

        let contract_addr = Self::env().account_id();
        ControllerRef::redeem_allowed(&self._controller(), contract_addr, redeemer, redeem_tokens)
            .unwrap();

        // TODO: assertion check - check current cash

        self._burn_from(redeemer, redeem_tokens).unwrap();
        PSP22Ref::transfer(
            &self._underlying(),
            redeemer,
            redeem_amount,
            Vec::<u8>::new(),
        )
        .unwrap();

        self._emit_redeem_event(redeemer, redeem_amount, redeem_tokens);

        Ok(())
    }
    default fn _borrow(&mut self, borrower: AccountId, borrow_amount: Balance) -> Result<()> {
        let contract_addr = Self::env().account_id();
        ControllerRef::borrow_allowed(&self._controller(), contract_addr, borrower, borrow_amount)
            .unwrap();

        // TODO: assertion check - compare current block number with accrual block number
        // TODO: assertion check - check current cash

        let account_borrows_prev = self._borrow_balance_stored(borrower);
        let account_borrows_new = account_borrows_prev + borrow_amount;
        let total_borrows_new = self._total_borrows() + borrow_amount;

        self.data::<Data>().account_borrows.insert(
            &borrower,
            &BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: 1, // TODO: borrow_index
            },
        );
        self.data::<Data>().total_borrows = total_borrows_new;

        PSP22Ref::transfer(
            &self._underlying(),
            borrower,
            borrow_amount,
            Vec::<u8>::new(),
        )
        .unwrap();

        self._emit_borrow_event(
            borrower,
            borrow_amount,
            account_borrows_new,
            total_borrows_new,
        );

        Ok(())
    }
    // NOTE: not working
    default fn _repay_borrow(
        &mut self,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        let contract_addr = Self::env().account_id();
        ControllerRef::repay_borrow_allowed(
            &self._controller(),
            contract_addr,
            payer,
            borrower,
            repay_amount,
        )
        .unwrap();

        // TODO: assertion check - compare current block number with accrual block number

        let account_borrow_prev = self._borrow_balance_stored(borrower);
        let repay_amount_final = if repay_amount == u128::MAX {
            account_borrow_prev
        } else {
            repay_amount
        };

        PSP22Ref::transfer_from_builder(
            &self._underlying(),
            payer,
            contract_addr,
            repay_amount_final,
            Vec::<u8>::new(),
        )
        .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))
        .try_invoke()
        .unwrap()
        .unwrap()
        .unwrap();

        let account_borrows_new = account_borrow_prev - repay_amount_final;
        let total_borrows_new = self._total_borrows() - repay_amount_final;

        self.data::<Data>().account_borrows.insert(
            &borrower,
            &BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: 1, // TODO: borrow_index
            },
        );
        self.data::<Data>().total_borrows = total_borrows_new;

        self._emit_repay_borrow_event(
            payer,
            borrower,
            repay_amount_final,
            account_borrows_new,
            total_borrows_new,
        );

        Ok(())
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

    default fn _underlying(&self) -> AccountId {
        self.data::<Data>().underlying
    }

    default fn _controller(&self) -> AccountId {
        self.data::<Data>().controller
    }

    default fn _get_cash_prior(&self) -> Balance {
        PSP22Ref::balance_of(&self._underlying(), Self::env().account_id())
    }

    default fn _total_borrows(&self) -> Balance {
        self.data::<Data>().total_borrows
    }

    default fn _borrow_balance_stored(&self, account: AccountId) -> Balance {
        let snapshot = match self.data::<Data>().account_borrows.get(&account) {
            Some(value) => value,
            None => return 0,
        };

        if snapshot.principal == 0 {
            return 0
        }
        let borrow_index = 1; // temp
        let prinicipal_times_index = snapshot.principal * borrow_index;
        return prinicipal_times_index / snapshot.interest_index
    }

    // event emission
    default fn _emit_mint_event(
        &self,
        _minter: AccountId,
        _mint_amount: Balance,
        _mint_tokens: Balance,
    ) {
    }
    default fn _emit_redeem_event(
        &self,
        _redeemer: AccountId,
        _redeem_amount: Balance,
        _redeem_tokens: Balance,
    ) {
    }
    default fn _emit_borrow_event(
        &self,
        _borrower: AccountId,
        _borrow_amount: Balance,
        _account_borrows: Balance,
        _total_borrows: Balance,
    ) {
    }
    default fn _emit_repay_borrow_event(
        &self,
        _payer: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
        _account_borrows: Balance,
        _total_borrows: Balance,
    ) {
    }
    default fn _emit_liquidate_borrow_event(
        &self,
        _liquidator: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
        _token_collateral: Balance,
        _seize_tokens: Balance,
    ) {
    }
}

pub fn to_psp22_error(e: psp22::PSP22Error) -> Error {
    Error::PSP22(e)
}

pub fn to_lang_error(e: LangError) -> Error {
    Error::Lang(e)
}
