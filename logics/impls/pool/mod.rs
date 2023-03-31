use super::{
    controller::{
        PoolAttributes,
        PoolAttributesForSeizeCalculation,
    },
    exp_no_err::{
        exp_scale,
        Exp,
    },
};
use crate::traits::{
    controller,
    types::WrappedU256,
};
pub use crate::traits::{
    controller::ControllerRef,
    interest_rate_model::InterestRateModelRef,
    pool::*,
};
use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};
use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp22::{
        self,
        extensions::metadata::PSP22Metadata,
        Data as PSP22Data,
        Internal as PSP22Internal,
        PSP22Error,
        PSP22Ref,
    },
    storage::Mapping,
    traits::{
        AccountId,
        Balance,
        Storage,
        String,
        Timestamp,
        ZERO_ADDRESS,
    },
};
use primitive_types::U256;

mod utils;
use self::utils::{
    calculate_interest,
    calculate_redeem_values,
    exchange_rate,
    protocol_seize_amount,
    protocol_seize_share_mantissa,
    reserve_factor_max_mantissa,
    underlying_balance,
    CalculateInterestInput,
    CalculateInterestOutput,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct BorrowSnapshot {
    principal: Balance,
    interest_index: Balance,
}

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub underlying: AccountId,
    pub controller: AccountId,
    pub manager: AccountId,
    pub rate_model: AccountId,
    pub total_borrows: Balance,
    pub total_reserves: Balance,
    pub account_borrows: Mapping<AccountId, BorrowSnapshot>,
    pub accural_block_timestamp: Timestamp,
    pub borrow_index: Balance,
    pub initial_exchange_rate_mantissa: WrappedU256,
    pub reserve_factor_mantissa: WrappedU256,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            underlying: ZERO_ADDRESS.into(),
            controller: ZERO_ADDRESS.into(),
            manager: ZERO_ADDRESS.into(),
            rate_model: ZERO_ADDRESS.into(),
            total_borrows: Default::default(),
            total_reserves: Default::default(),
            account_borrows: Default::default(),
            accural_block_timestamp: 0,
            borrow_index: 1,
            initial_exchange_rate_mantissa: WrappedU256::from(U256::zero()),
            reserve_factor_mantissa: WrappedU256::from(U256::zero()),
        }
    }
}

pub trait Internal {
    fn _accrue_interest(&mut self) -> Result<()>;
    fn _accure_interest_at(&mut self, at: Timestamp) -> Result<()>;
    fn _balance_of(&self, owner: &AccountId) -> Balance;
    fn _total_supply(&self) -> Balance;
    // use in PSP22#transfer,transfer_from interface
    // return PSP22Error as Error for this
    fn _transfer_tokens(
        &mut self,
        spender: AccountId,
        src: AccountId,
        dst: AccountId,
        value: Balance,
        data: Vec<u8>,
    ) -> core::result::Result<(), PSP22Error>;
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
    ) -> Result<Balance>;
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
        seize_tokens: Balance,
    ) -> Result<()>;

    // admin functions
    fn _set_controller(&mut self, new_controller: AccountId) -> Result<()>;
    fn _set_reserve_factor_mantissa(
        &mut self,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()>;
    fn _set_interest_rate_model(&mut self, new_interest_rate_model: AccountId) -> Result<()>;
    fn _add_reserves(&mut self, amount: Balance) -> Result<()>;
    fn _reduce_reserves(&mut self, admin: AccountId, amount: Balance) -> Result<()>;
    fn _sweep_token(&mut self, asset: AccountId) -> Result<()>;

    // utilities
    fn _transfer_underlying_from(
        &self,
        from: AccountId,
        to: AccountId,
        value: Balance,
    ) -> Result<()>;
    fn _transfer_underlying(&self, to: AccountId, value: Balance) -> Result<()>;
    fn _assert_manager(&self) -> Result<()>;

    // view functions
    fn _underlying(&self) -> AccountId;
    fn _controller(&self) -> AccountId;
    fn _manager(&self) -> AccountId;
    fn _get_cash_prior(&self) -> Balance;
    fn _total_borrows(&self) -> Balance;
    fn _total_reserves(&self) -> Balance;
    fn _rate_model(&self) -> AccountId;
    fn _borrow_rate_per_msec(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> WrappedU256;
    fn _supply_rate_per_msec(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor: WrappedU256,
    ) -> WrappedU256;
    fn _borrow_balance_stored(&self, account: AccountId) -> Balance;
    fn _balance_of_underlying(&self, account: AccountId) -> Balance;
    fn _principal_balance_of(&self, account: &AccountId) -> Balance;
    fn _accural_block_timestamp(&self) -> Timestamp;
    fn _borrow_index(&self) -> Balance;
    fn _initial_exchange_rate_mantissa(&self) -> WrappedU256;
    fn _reserve_factor_mantissa(&self) -> WrappedU256;
    fn _exchange_rate_stored(&self) -> U256;
    fn _get_interest_at(&self, at: Timestamp) -> Result<CalculateInterestOutput>;

    // event emission
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
        token_collateral: AccountId,
        seize_tokens: Balance,
    );
    fn _emit_accrue_interest_event(
        &self,
        interest_accumulated: Balance,
        new_index: Balance,
        new_total_borrows: Balance,
    );
    fn _emit_reserves_added_event(
        &self,
        benefactor: AccountId,
        add_amount: Balance,
        new_total_reserves: Balance,
    );
    fn _emit_reserves_reduced_event(&self, reduce_amount: Balance, total_reserves_new: Balance);
    fn _emit_new_controller_event(&self, old: AccountId, new: AccountId);
    fn _emit_new_interest_rate_model_event(&self, old: AccountId, new: AccountId);
    fn _emit_new_reserve_factor_event(&self, old: WrappedU256, new: WrappedU256);
}

impl<T: Storage<Data> + Storage<psp22::Data> + Storage<psp22::extensions::metadata::Data>> Pool
    for T
{
    default fn accrue_interest(&mut self) -> Result<()> {
        self._accrue_interest()
    }

    default fn mint(&mut self, mint_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._mint(Self::env().caller(), mint_amount)
    }

    default fn get_accrual_block_timestamp(&self) -> Timestamp {
        self._accural_block_timestamp()
    }

    default fn redeem(&mut self, redeem_tokens: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._redeem(Self::env().caller(), redeem_tokens, 0)
    }

    default fn redeem_underlying(&mut self, redeem_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._redeem(Self::env().caller(), 0, redeem_amount)
    }

    default fn borrow(&mut self, borrow_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._borrow(Self::env().caller(), borrow_amount)
    }

    default fn repay_borrow(&mut self, repay_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._repay_borrow(Self::env().caller(), Self::env().caller(), repay_amount)?;
        Ok(())
    }

    default fn repay_borrow_behalf(
        &mut self,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        self._accrue_interest()?;
        self._repay_borrow(Self::env().caller(), borrower, repay_amount)?;
        Ok(())
    }

    default fn liquidate_borrow(
        &mut self,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()> {
        self._accrue_interest()?;
        if collateral != Self::env().account_id() {
            PoolRef::accrue_interest(&collateral)?;
        }
        self._liquidate_borrow(Self::env().caller(), borrower, repay_amount, collateral)
    }

    default fn seize(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()> {
        self._accrue_interest()?;
        self._seize(Self::env().caller(), liquidator, borrower, seize_tokens)
    }

    default fn set_controller(&mut self, new_controller: AccountId) -> Result<()> {
        self._assert_manager()?;
        let old = self._controller();
        self._set_controller(new_controller)?;
        self._emit_new_controller_event(old, new_controller);
        Ok(())
    }

    default fn set_reserve_factor_mantissa(
        &mut self,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        let old = self._reserve_factor_mantissa();
        self._set_reserve_factor_mantissa(new_reserve_factor_mantissa)?;
        self._emit_new_reserve_factor_event(old, new_reserve_factor_mantissa);
        Ok(())
    }

    default fn set_interest_rate_model(
        &mut self,
        new_interest_rate_model: AccountId,
    ) -> Result<()> {
        self._assert_manager()?;
        let old = self._rate_model();
        self._set_interest_rate_model(new_interest_rate_model)?;
        self._emit_new_interest_rate_model_event(old, new_interest_rate_model);
        Ok(())
    }

    default fn add_reserves(&mut self, amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._add_reserves(amount)
    }

    default fn reduce_reserves(&mut self, amount: Balance) -> Result<()> {
        self._assert_manager()?;
        self._accrue_interest()?;
        self._reduce_reserves(Self::env().caller(), amount)
    }

    default fn sweep_token(&mut self, asset: AccountId) -> Result<()> {
        self._assert_manager()?;
        self._sweep_token(asset)
    }

    default fn underlying(&self) -> AccountId {
        self._underlying()
    }

    default fn controller(&self) -> AccountId {
        self._controller()
    }

    default fn manager(&self) -> AccountId {
        self._manager()
    }

    default fn exchage_rate_stored(&self) -> WrappedU256 {
        WrappedU256::from(self._exchange_rate_stored())
    }

    default fn exchange_rate_current(&mut self) -> Result<WrappedU256> {
        self._accrue_interest()?;
        Ok(self.exchage_rate_stored())
    }

    default fn get_cash_prior(&self) -> Balance {
        self._get_cash_prior()
    }

    default fn total_borrows(&self) -> Balance {
        self._total_borrows()
    }

    default fn total_reserves(&self) -> Balance {
        self._total_reserves()
    }

    default fn get_account_snapshot(&self, account: AccountId) -> (Balance, Balance, U256) {
        (
            Internal::_balance_of(self, &account),
            self._borrow_balance_stored(account),
            self._exchange_rate_stored(),
        )
    }

    default fn borrow_balance_stored(&self, account: AccountId) -> Balance {
        self._borrow_balance_stored(account)
    }

    default fn borrow_balance_current(&mut self, account: AccountId) -> Result<Balance> {
        self._accrue_interest()?;
        Ok(self._borrow_balance_stored(account))
    }

    default fn borrow_rate_per_msec(&self) -> WrappedU256 {
        let cash = self._get_cash_prior();
        let borrows = self._total_borrows();
        let reserves = self._total_reserves();
        self._borrow_rate_per_msec(cash, borrows, reserves)
    }

    default fn supply_rate_per_msec(&self) -> WrappedU256 {
        let cash = self._get_cash_prior();
        let borrows = self._total_borrows();
        let reserves = self._total_reserves();
        let reserve_factor = self._reserve_factor_mantissa();
        self._supply_rate_per_msec(cash, borrows, reserves, reserve_factor)
    }

    default fn principal_balance_of(&self, account: AccountId) -> Balance {
        self._principal_balance_of(&account)
    }

    default fn initial_exchange_rate_mantissa(&self) -> WrappedU256 {
        self._initial_exchange_rate_mantissa()
    }

    default fn reserve_factor_mantissa(&self) -> WrappedU256 {
        self._reserve_factor_mantissa()
    }
}

impl<T: Storage<Data> + Storage<psp22::Data> + Storage<psp22::extensions::metadata::Data>> Internal
    for T
{
    default fn _accrue_interest(&mut self) -> Result<()> {
        self._accure_interest_at(Self::env().block_timestamp())
    }
    default fn _accure_interest_at(&mut self, at: Timestamp) -> Result<()> {
        let accural = self._accural_block_timestamp();
        if accural.eq(&at) {
            return Ok(())
        }

        let out = self._get_interest_at(at)?;
        let mut data = self.data::<Data>();
        data.accural_block_timestamp = at;
        data.borrow_index = out.borrow_index;
        data.total_borrows = out.total_borrows;
        data.total_reserves = out.total_reserves;
        self._emit_accrue_interest_event(
            out.interest_accumulated,
            out.borrow_index,
            out.total_borrows,
        );
        Ok(())
    }

    fn _get_interest_at(&self, at: Timestamp) -> Result<CalculateInterestOutput> {
        let cash = self._get_cash_prior();
        let borrows = self._total_borrows();
        let reserves = self._total_reserves();
        let idx = self._borrow_index();
        let borrow_rate =
            InterestRateModelRef::get_borrow_rate(&self._rate_model(), cash, borrows, reserves);
        calculate_interest(&CalculateInterestInput {
            total_borrows: borrows,
            total_reserves: reserves,
            borrow_index: idx,
            borrow_rate: borrow_rate.into(),
            old_block_timestamp: self._accural_block_timestamp(),
            new_block_timestamp: at,
            reserve_factor_mantissa: self._reserve_factor_mantissa().into(),
        })
    }
    default fn _transfer_tokens(
        &mut self,
        spender: AccountId,
        src: AccountId,
        dst: AccountId,
        value: Balance,
        data: Vec<u8>,
    ) -> core::result::Result<(), PSP22Error> {
        let contract_addr = Self::env().account_id();
        let (account_balance, account_borrow_balance, exchange_rate) =
            self.get_account_snapshot(src);
        let pool_attribute = PoolAttributes {
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
        };
        ControllerRef::transfer_allowed(
            &self._controller(),
            contract_addr,
            src,
            dst,
            value,
            Some(pool_attribute),
        )?;

        if src == dst {
            return Err(PSP22Error::Custom(String::from("TransferNotAllowed")))
        }

        if spender == src {
            // copied from PSP22#transfer
            // ref: https://github.com/727-Ventures/openbrush-contracts/blob/868ee023727c49296b774327bee25db7b5160c49/contracts/src/token/psp22/psp22.rs#L75-L79
            self._transfer_from_to(src, dst, value, data)?;
        } else {
            // copied from PSP22#transfer_from
            // ref: https://github.com/727-Ventures/openbrush-contracts/blob/868ee023727c49296b774327bee25db7b5160c49/contracts/src/token/psp22/psp22.rs#L81-L98
            let allowance = self._allowance(&src, &spender);

            if allowance < value {
                return Err(PSP22Error::InsufficientAllowance)
            }

            self._approve_from_to(src, spender, allowance - value)?;
            self._transfer_from_to(src, dst, value, data)?;
        }

        Ok(())
    }
    default fn _mint(&mut self, minter: AccountId, mint_amount: Balance) -> Result<()> {
        let contract_addr = Self::env().account_id();
        ControllerRef::mint_allowed(&self._controller(), contract_addr, minter, mint_amount)?;

        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        };

        let exchange_rate = self._exchange_rate_stored(); // NOTE: need exchange_rate calculation before transfer underlying
        self._transfer_underlying_from(minter, contract_addr, mint_amount)?;
        let minted_tokens = U256::from(mint_amount)
            .mul(exp_scale())
            .div(exchange_rate)
            .as_u128();
        self._mint_to(minter, minted_tokens)?;

        self._emit_mint_event(minter, mint_amount, minted_tokens);

        // skip post-process because nothing is done
        // ControllerRef::mint_verify(&self._controller(), contract_addr, minter, minted_amount, mint_amount)?;

        Ok(())
    }
    default fn _redeem(
        &mut self,
        redeemer: AccountId,
        redeem_tokens_in: Balance,
        redeem_amount_in: Balance,
    ) -> Result<()> {
        let values = calculate_redeem_values(
            redeem_tokens_in,
            redeem_amount_in,
            self._exchange_rate_stored(),
        );
        if values.is_none() {
            return Err(Error::InvalidParameter)
        }
        let (redeem_tokens, redeem_amount) = values.unwrap();
        if (redeem_tokens == 0 && redeem_amount > 0) || (redeem_tokens > 0 && redeem_amount == 0) {
            return Err(Error::OnlyEitherRedeemTokensOrRedeemAmountIsZero)
        }

        let contract_addr = Self::env().account_id();
        let (account_balance, account_borrow_balance, exchange_rate) =
            self.get_account_snapshot(redeemer);
        let pool_attribute = PoolAttributes {
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
        };
        ControllerRef::redeem_allowed(
            &self._controller(),
            contract_addr,
            redeemer,
            redeem_tokens,
            Some(pool_attribute),
        )?;
        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        if self._get_cash_prior() < redeem_amount {
            return Err(Error::RedeemTransferOutNotPossible)
        }

        self._burn_from(redeemer, redeem_tokens)?;
        self._transfer_underlying(redeemer, redeem_amount)?;

        self._emit_redeem_event(redeemer, redeem_amount, redeem_tokens);

        // skip post-process because nothing is done
        // ControllerRef::redeem_verify(&self._controller(), contract_addr, redeemer, redeem_tokens, redeem_amount)?;

        Ok(())
    }
    default fn _borrow(&mut self, borrower: AccountId, borrow_amount: Balance) -> Result<()> {
        let contract_addr = Self::env().account_id();
        let (account_balance, account_borrow_balance, exchange_rate) =
            self.get_account_snapshot(borrower);
        let pool_attribute = PoolAttributes {
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
        };
        ControllerRef::borrow_allowed(
            &self._controller(),
            contract_addr,
            borrower,
            borrow_amount,
            Some(pool_attribute),
        )?;

        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        };
        if self._get_cash_prior() < borrow_amount {
            return Err(Error::BorrowCashNotAvailable)
        }

        let account_borrows_prev = self._borrow_balance_stored(borrower);
        let account_borrows_new = account_borrows_prev + borrow_amount;
        let total_borrows_new = self._total_borrows() + borrow_amount;
        let idx = self._borrow_index();
        self.data::<Data>().account_borrows.insert(
            &borrower,
            &BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: idx,
            },
        );
        self.data::<Data>().total_borrows = total_borrows_new;

        self._transfer_underlying(borrower, borrow_amount)?;

        self._emit_borrow_event(
            borrower,
            borrow_amount,
            account_borrows_new,
            total_borrows_new,
        );

        // skip post-process because nothing is done
        // ControllerRef::borrow_verify(&self._controller(), contract_addr, borrower, borrow_amount)?;

        Ok(())
    }

    default fn _repay_borrow(
        &mut self,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<Balance> {
        let contract_addr = Self::env().account_id();
        ControllerRef::repay_borrow_allowed(
            &self._controller(),
            contract_addr,
            payer,
            borrower,
            repay_amount,
        )?;

        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        };

        let account_borrow_prev = self._borrow_balance_stored(borrower);
        let repay_amount_final = if repay_amount == u128::MAX {
            account_borrow_prev
        } else {
            repay_amount
        };

        self._transfer_underlying_from(payer, contract_addr, repay_amount_final)?;

        let account_borrows_new = account_borrow_prev - repay_amount_final;
        let total_borrows_new = self._total_borrows() - repay_amount_final;

        let idx = self._borrow_index();
        self.data::<Data>().account_borrows.insert(
            &borrower,
            &BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: idx,
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

        // skip post-process because nothing is done
        // ControllerRef::repay_borrow_verify(&self._controller(), contract_addr, payer, borrower, repay_amount_final, 0)?; // temp: index is zero (type difference)

        Ok(repay_amount_final)
    }
    default fn _liquidate_borrow(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()> {
        let contract_addr = Self::env().account_id();
        let (account_balance, account_borrow_balance, exchange_rate) =
            self.get_account_snapshot(borrower);
        let pool_attribute = PoolAttributes {
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
        };
        ControllerRef::liquidate_borrow_allowed(
            &self._controller(),
            contract_addr,
            collateral,
            liquidator,
            borrower,
            repay_amount,
            Some(pool_attribute),
        )?;

        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }
        if collateral != contract_addr {
            if PoolRef::get_accrual_block_timestamp(&collateral) != current_timestamp {
                return Err(Error::AccrualBlockNumberIsNotFresh)
            }
        }
        if liquidator == borrower {
            return Err(Error::LiquidateLiquidatorIsBorrower)
        }
        if repay_amount == 0 {
            return Err(Error::LiquidateCloseAmountIsZero)
        }

        let actual_repay_amount = self._repay_borrow(liquidator, borrower, repay_amount)?;
        let pool_borrowed_attributes = Some(PoolAttributesForSeizeCalculation {
            underlying: self._underlying(),
            decimals: self.token_decimals(),
        });
        let seize_tokens = if collateral == contract_addr {
            let pool_collateral_attributes = pool_borrowed_attributes.clone();
            let seize_tokens = ControllerRef::liquidate_calculate_seize_tokens(
                &self._controller(),
                contract_addr,
                collateral,
                WrappedU256::from(self._exchange_rate_stored()),
                actual_repay_amount,
                pool_borrowed_attributes,
                pool_collateral_attributes,
            )?;

            self._seize(contract_addr, liquidator, borrower, seize_tokens)?;

            seize_tokens
        } else {
            let seize_tokens = ControllerRef::liquidate_calculate_seize_tokens(
                &self._controller(),
                contract_addr,
                collateral,
                PoolRef::exchage_rate_stored(&collateral),
                actual_repay_amount,
                pool_borrowed_attributes,
                Some(PoolAttributesForSeizeCalculation {
                    underlying: PoolRef::underlying(&collateral),
                    decimals: PoolRef::token_decimals(&collateral),
                }),
            )?;

            PoolRef::seize(&collateral, liquidator, borrower, seize_tokens)?;

            seize_tokens
        };

        self._emit_liquidate_borrow_event(
            liquidator,
            borrower,
            actual_repay_amount,
            collateral,
            seize_tokens,
        );

        // skip post-process because nothing is done
        // ControllerRef::liquidate_borrow_verify(&self._controller(), contract_addr, collateral, liquidator, borrower, actual_repay_amount, seize_tokens)?;

        Ok(())
    }
    default fn _seize(
        &mut self,
        seizer_token: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()> {
        let contract_addr = Self::env().account_id();
        ControllerRef::seize_allowed(
            &self._controller(),
            contract_addr,
            seizer_token,
            liquidator,
            borrower,
            seize_tokens,
        )?;

        if liquidator == borrower {
            return Err(Error::LiquidateSeizeLiquidatorIsBorrower)
        }
        // calculate the new borrower and liquidator token balances
        let exchange_rate = Exp {
            mantissa: WrappedU256::from(self._exchange_rate_stored()),
        };
        let (liquidator_seize_tokens, protocol_seize_amount, protocol_seize_tokens) =
            protocol_seize_amount(exchange_rate, seize_tokens, protocol_seize_share_mantissa());
        let total_reserves_new = self._total_reserves() + protocol_seize_amount;

        // EFFECTS & INTERACTIONS
        self.data::<Data>().total_reserves = total_reserves_new;
        self.data::<PSP22Data>().supply -= protocol_seize_tokens;
        self._burn_from(borrower, seize_tokens)?;
        self._mint_to(liquidator, liquidator_seize_tokens)?;

        self._emit_reserves_added_event(contract_addr, protocol_seize_amount, total_reserves_new);

        // skip post-process because nothing is done
        // ControllerRef::seize_verify(&self._controller(), contract_addr, seizer_token, liquidator, borrower, seize_tokens)?;

        Ok(())
    }

    // admin functions
    default fn _set_controller(&mut self, new_controller: AccountId) -> Result<()> {
        self.data::<Data>().controller = new_controller;
        Ok(())
    }

    default fn _set_reserve_factor_mantissa(
        &mut self,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._accrue_interest()?;

        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        if U256::from(new_reserve_factor_mantissa).gt(&reserve_factor_max_mantissa()) {
            return Err(Error::SetReserveFactorBoundsCheck)
        }

        self.data::<Data>().reserve_factor_mantissa = new_reserve_factor_mantissa;
        Ok(())
    }

    default fn _set_interest_rate_model(
        &mut self,
        new_interest_rate_model: AccountId,
    ) -> Result<()> {
        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        self.data::<Data>().rate_model = new_interest_rate_model;
        Ok(())
    }

    default fn _add_reserves(&mut self, amount: Balance) -> Result<()> {
        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        let total_reserves_new = self._total_reserves().add(amount);
        self.data::<Data>().total_reserves = total_reserves_new;
        let caller = Self::env().caller();
        self._transfer_underlying_from(caller, Self::env().account_id(), amount)?;

        // event
        self._emit_reserves_added_event(caller, amount, total_reserves_new);

        Ok(())
    }

    default fn _reduce_reserves(&mut self, admin: AccountId, amount: Balance) -> Result<()> {
        let current_timestamp = Self::env().block_timestamp();
        if self._accural_block_timestamp() != current_timestamp {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        if self._get_cash_prior().lt(&amount) {
            return Err(Error::ReduceReservesCashNotAvailable)
        }
        if self._total_reserves().lt(&amount) {
            return Err(Error::ReduceReservesCashValidation)
        }
        let total_reserves_new = self._total_reserves().sub(amount);
        let mut data = self.data::<Data>();
        data.total_reserves = total_reserves_new;
        self._transfer_underlying(admin, amount)?;

        // event
        self._emit_reserves_reduced_event(amount, total_reserves_new);
        Ok(())
    }

    default fn _sweep_token(&mut self, asset: AccountId) -> Result<()> {
        if asset == self._underlying() {
            return Err(Error::CannotSweepUnderlyingToken)
        }

        let balance = PSP22Ref::balance_of(&asset, Self::env().account_id());
        PSP22Ref::transfer(&asset, Self::env().caller(), balance, Vec::<u8>::new())?;
        Ok(())
    }

    // utilities
    default fn _transfer_underlying_from(
        &self,
        from: AccountId,
        to: AccountId,
        value: Balance,
    ) -> Result<()> {
        PSP22Ref::transfer_from_builder(&self._underlying(), from, to, value, Vec::<u8>::new())
            .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))
            .try_invoke()
            .unwrap()
            .unwrap()
            .map_err(to_psp22_error)
    }

    default fn _transfer_underlying(&self, to: AccountId, value: Balance) -> Result<()> {
        PSP22Ref::transfer(&self._underlying(), to, value, Vec::<u8>::new()).map_err(to_psp22_error)
    }

    default fn _assert_manager(&self) -> Result<()> {
        if Self::env().caller() != self._manager() {
            return Err(Error::CallerIsNotManager)
        }
        Ok(())
    }

    // view functions
    default fn _underlying(&self) -> AccountId {
        self.data::<Data>().underlying
    }

    default fn _controller(&self) -> AccountId {
        self.data::<Data>().controller
    }

    default fn _manager(&self) -> AccountId {
        self.data::<Data>().manager
    }

    default fn _get_cash_prior(&self) -> Balance {
        PSP22Ref::balance_of(&self._underlying(), Self::env().account_id())
    }

    default fn _total_borrows(&self) -> Balance {
        self.data::<Data>().total_borrows
    }

    default fn _rate_model(&self) -> AccountId {
        self.data::<Data>().rate_model
    }

    default fn _borrow_rate_per_msec(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> WrappedU256 {
        InterestRateModelRef::get_borrow_rate(&self._rate_model(), cash, borrows, reserves)
    }

    default fn _supply_rate_per_msec(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: WrappedU256,
    ) -> WrappedU256 {
        InterestRateModelRef::get_supply_rate(
            &self._rate_model(),
            cash,
            borrows,
            reserves,
            reserve_factor_mantissa,
        )
    }

    default fn _accural_block_timestamp(&self) -> Timestamp {
        Timestamp::from(self.data::<Data>().accural_block_timestamp)
    }

    default fn _total_reserves(&self) -> Balance {
        self.data::<Data>().total_reserves
    }

    default fn _borrow_index(&self) -> Balance {
        self.data::<Data>().borrow_index
    }

    default fn _borrow_balance_stored(&self, account: AccountId) -> Balance {
        let snapshot = match self.data::<Data>().account_borrows.get(&account) {
            Some(value) => value,
            None => return 0,
        };

        if snapshot.principal == 0 {
            return 0
        }
        let borrow_index = self._borrow_index();
        let prinicipal_times_index = U256::from(snapshot.principal).mul(U256::from(borrow_index));
        prinicipal_times_index
            .div(U256::from(snapshot.interest_index))
            .as_u128()
    }

    default fn _balance_of(&self, owner: &AccountId) -> Balance {
        self._balance_of_underlying(*owner)
    }

    default fn _total_supply(&self) -> Balance {
        let supply = self.data::<PSP22Data>().supply;
        let interest = self
            ._get_interest_at(Self::env().block_timestamp())
            .unwrap();
        let rate = exchange_rate(
            supply.into(),
            self._get_cash_prior(),
            interest.total_borrows,
            interest.total_reserves,
            U256::from(self._initial_exchange_rate_mantissa()),
        );
        underlying_balance(
            Exp {
                mantissa: rate.into(),
            },
            supply,
        )
    }

    default fn _balance_of_underlying(&self, account: AccountId) -> Balance {
        let exchange_rate = Exp {
            mantissa: self._exchange_rate_stored().into(),
        };
        let pool_token_balance = self._principal_balance_of(&account);
        underlying_balance(exchange_rate, pool_token_balance)
    }

    default fn _principal_balance_of(&self, account: &AccountId) -> Balance {
        psp22::Internal::_balance_of(self, account)
    }

    default fn _initial_exchange_rate_mantissa(&self) -> WrappedU256 {
        self.data::<Data>().initial_exchange_rate_mantissa
    }

    default fn _reserve_factor_mantissa(&self) -> WrappedU256 {
        self.data::<Data>().reserve_factor_mantissa
    }

    default fn _exchange_rate_stored(&self) -> U256 {
        exchange_rate(
            self.data::<PSP22Data>().supply,
            self._get_cash_prior(),
            self._total_borrows(),
            self._total_reserves(),
            U256::from(self._initial_exchange_rate_mantissa()),
        )
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
        _token_collateral: AccountId,
        _seize_tokens: Balance,
    ) {
    }

    default fn _emit_reserves_added_event(
        &self,
        _benefactor: AccountId,
        _add_amount: Balance,
        _new_total_reserves: Balance,
    ) {
    }

    default fn _emit_accrue_interest_event(
        &self,
        _interest_accumulated: Balance,
        _new_index: Balance,
        _new_total_borrows: Balance,
    ) {
    }

    default fn _emit_reserves_reduced_event(
        &self,
        _reduce_amount: Balance,
        _total_reserves_new: Balance,
    ) {
    }

    default fn _emit_new_controller_event(&self, _old: AccountId, _new: AccountId) {}
    default fn _emit_new_interest_rate_model_event(&self, _old: AccountId, _new: AccountId) {}
    default fn _emit_new_reserve_factor_event(&self, _old: WrappedU256, _new: WrappedU256) {}
}

pub fn to_psp22_error(e: PSP22Error) -> Error {
    Error::PSP22(e)
}

impl From<controller::Error> for PSP22Error {
    fn from(error: controller::Error) -> Self {
        let convert = { |str: &str| PSP22Error::Custom(String::from(str)) };
        return match error {
            controller::Error::MintIsPaused => convert("MintIsPaused"),
            controller::Error::BorrowIsPaused => convert("BorrowIsPaused"),
            controller::Error::SeizeIsPaused => convert("SeizeIsPaused"),
            controller::Error::TransferIsPaused => convert("TransferIsPaused"),
            controller::Error::MarketNotListed => convert("MarketNotListed"),
            controller::Error::MarketAlreadyListed => convert("MarketAlreadyListed"),
            controller::Error::ControllerMismatch => convert("ControllerMismatch"),
            controller::Error::PriceError => convert("PriceError"),
            controller::Error::TooMuchRepay => convert("TooMuchRepay"),
            controller::Error::BorrowCapReached => convert("BorrowCapReached"),
            controller::Error::InsufficientLiquidity => convert("InsufficientLiquidity"),
            controller::Error::InsufficientShortfall => convert("InsufficientShortfall"),
            controller::Error::CallerIsNotManager => convert("CallerIsNotManager"),
            controller::Error::InvalidCollateralFactor => convert("InvalidCollateralFactor"),
        }
    }
}
