// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
    incentives_controller::IncentivesControllerRef,
    types::WrappedU256,
};
pub use crate::traits::{
    controller::{
        ControllerRef,
        Error as ControllerError,
    },
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
    modifier_definition,
    modifiers,
    storage::{
        Mapping,
        TypeGuard,
    },
    traits::{
        AccountId,
        AccountIdExt,
        Balance,
        BlockNumber,
        Storage,
        String,
    },
};
use primitive_types::U256;

pub mod utils;
use self::utils::{
    calculate_interest,
    exchange_rate,
    protocol_seize_amount,
    protocol_seize_share_mantissa,
    reserve_factor_max_mantissa,
    underlying_balance,
    CalculateInterestInput,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const COLLATERAL_FACTOR_MANTISSA_DECIMALS: u32 = 18;
pub const LIQUIDATION_THRESHOLD_DECIMALS: u32 = 4;

#[derive(Debug, scale::Decode, scale::Encode, Default)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct BorrowSnapshot {
    principal: Balance,
    interest_index: WrappedU256,
}

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// AccountId of underlying asset
    pub underlying: Option<AccountId>,
    /// AccountId of Controller managing this pool
    pub controller: Option<AccountId>,
    /// AccountId of Manager, the administrator of this pool
    pub manager: Option<AccountId>,
    /// AccountId of Pending Manager use for transfer manager role
    pub pending_manager: Option<AccountId>,
    /// AccountId of incentives conroller
    pub incentives_controller: Option<AccountId>,
    /// AccountId of Rate Model
    pub rate_model: Option<AccountId>,
    /// Total borrows
    pub total_borrows: Balance,
    /// Total reserves
    pub total_reserves: Balance,
    /// Borrow balance for accounts
    pub account_borrows: Mapping<AccountId, BorrowSnapshot>,
    /// Last block number of interest calculation process execution
    pub accrual_block_number: BlockNumber,
    /// Borrow index for interests
    pub borrow_index: WrappedU256,
    /// Initial exchange_rate, Used if never called
    pub initial_exchange_rate_mantissa: WrappedU256,
    /// Maximum fraction of interest that can be set aside for reserves
    pub reserve_factor_mantissa: WrappedU256,
    /// Liquidation Threshold (Decimals: 4)
    pub liquidation_threshold: u128,
    /// Delegation Allowance for borrowing
    pub delegate_allowance: Mapping<(AccountId, AccountId), Balance, AllowancesKey>,
    /// Represent if user is using his reserve as collateral or not
    pub using_reserve_as_collateral: Mapping<AccountId, bool>,
}

pub struct AllowancesKey;

impl<'a> TypeGuard<'a> for AllowancesKey {
    type Type = &'a (&'a AccountId, &'a AccountId);
}

impl Default for Data {
    fn default() -> Self {
        Data {
            underlying: None,
            controller: None,
            manager: None,
            pending_manager: None,
            rate_model: None,
            incentives_controller: None,
            total_borrows: Default::default(),
            total_reserves: Default::default(),
            account_borrows: Default::default(),
            delegate_allowance: Default::default(),
            accrual_block_number: 0,
            borrow_index: exp_scale().into(),
            initial_exchange_rate_mantissa: WrappedU256::from(U256::zero()),
            reserve_factor_mantissa: WrappedU256::from(U256::zero()),
            liquidation_threshold: 10000,
            using_reserve_as_collateral: Default::default(),
        }
    }
}

pub trait Internal {
    fn _accrue_interest(&mut self) -> Result<()>;
    fn _accrue_interest_at(&mut self, at: BlockNumber) -> Result<()>;

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
    fn _redeem(&mut self, redeemer: AccountId, amount: Balance) -> Result<()>;
    fn _borrow(
        &mut self,
        borrower: AccountId,
        borrow_amount: Balance,
        release_underlying: bool,
    ) -> Result<()>;
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
    fn _set_liquidation_threshold(&mut self, new_liquidation_threshold: u128) -> Result<()>;
    fn _approve_delegate(
        &mut self,
        owner: AccountId,
        delegatee: AccountId,
        amount: Balance,
    ) -> Result<()>;
    fn _set_use_reserve_as_collateral(&mut self, user: AccountId, use_as_collateral: bool);
    // utilities
    fn _transfer_underlying_from(
        &self,
        from: AccountId,
        to: AccountId,
        value: Balance,
    ) -> Result<()>;
    fn _transfer_underlying(&self, to: AccountId, value: Balance) -> Result<()>;
    fn _assert_manager(&self) -> Result<()>;
    fn _assert_pending_manager(&self) -> Result<()>;
    fn _validate_set_use_reserve_as_collateral(
        &mut self,
        user: AccountId,
        use_as_collateral: bool,
    ) -> Result<()>;
    fn _accrue_reward(&self, user: AccountId) -> Result<()>;
    fn _set_incentives_controller(&mut self, incentives_controller: AccountId) -> Result<()>;
    fn _set_manager(&mut self, manager: AccountId) -> Result<()>;
    fn _accept_manager(&mut self) -> Result<()>;
    // view functions
    fn _underlying(&self) -> Option<AccountId>;
    fn _controller(&self) -> Option<AccountId>;
    fn _manager(&self) -> Option<AccountId>;
    fn _pending_manager(&self) -> Option<AccountId>;
    fn _incentives_controller(&self) -> Option<AccountId>;
    fn _get_cash_prior(&self) -> Balance;
    fn _total_borrows(&self) -> Balance;
    fn _total_reserves(&self) -> Balance;
    fn _rate_model(&self) -> Option<AccountId>;
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
    fn _accrual_block_number(&self) -> BlockNumber;
    fn _borrow_index(&self) -> WrappedU256;
    fn _initial_exchange_rate_mantissa(&self) -> WrappedU256;
    fn _reserve_factor_mantissa(&self) -> WrappedU256;
    fn _exchange_rate_stored(&self) -> U256;
    fn _liquidation_threshold(&self) -> u128;
    fn _delegate_allowance(&self, owner: &AccountId, delegatee: &AccountId) -> Balance;
    fn _using_reserve_as_collateral(&self, user: AccountId) -> Option<bool>;
    // event emission
    fn _emit_mint_event(&self, minter: AccountId, mint_amount: Balance, mint_tokens: Balance);
    fn _emit_redeem_event(&self, redeemer: AccountId, redeem_amount: Balance);
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
        cash_prior: Balance,
        interest_accumulated: Balance,
        new_index: WrappedU256,
        new_total_borrows: Balance,
    );
    fn _emit_reserves_added_event(
        &self,
        benefactor: AccountId,
        add_amount: Balance,
        new_total_reserves: Balance,
    );
    fn _emit_reserves_reduced_event(&self, reduce_amount: Balance, total_reserves_new: Balance);
    fn _emit_new_controller_event(&self, old: Option<AccountId>, new: Option<AccountId>);
    fn _emit_new_interest_rate_model_event(&self, old: Option<AccountId>, new: Option<AccountId>);
    fn _emit_new_reserve_factor_event(&self, old: WrappedU256, new: WrappedU256);
    fn _emit_delegate_approval_event(
        &self,
        owner: AccountId,
        delegatee: AccountId,
        amount: Balance,
    );
    fn _emit_reserve_used_as_collateral_enabled_event(&self, user: AccountId);
    fn _emit_reserve_used_as_collateral_disabled_event(&self, user: AccountId);
    fn _emit_manager_updated_event(&self, old: AccountId, new: AccountId);
}

#[modifier_definition]
pub fn delegated_allowed<T, F, R>(
    instance: &mut T,
    body: F,
    owner: AccountId,
    amount: Balance,
) -> Result<R>
where
    T: Storage<Data> + Storage<psp22::Data> + Storage<psp22::extensions::metadata::Data>,
    F: FnOnce(&mut T) -> Result<R>,
{
    let delegatee = T::env().caller();
    if delegatee != owner {
        let delegate_allowance = instance._delegate_allowance(&owner, &delegatee);
        if delegate_allowance < amount {
            return Err(Error::InsufficientDelegateAllowance)
        }
    }
    body(instance)
}

#[modifier_definition]
pub fn only_flashloan_gateway<T, F, R>(instance: &mut T, body: F) -> Result<R>
where
    T: Storage<Data> + Storage<psp22::Data> + Storage<psp22::extensions::metadata::Data>,
    F: FnOnce(&mut T) -> Result<R>,
{
    let caller = T::env().caller();
    let controller = instance._controller().ok_or(Error::ControllerIsNotSet)?;
    let flashloan_gateway =
        ControllerRef::flashloan_gateway(&controller).ok_or(Error::CallerIsNotFlashloanGateway)?;
    if caller != flashloan_gateway {
        return Err(Error::CallerIsNotFlashloanGateway)
    }

    body(instance)
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

    default fn mint_to(&mut self, mint_account: AccountId, mint_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._mint(mint_account, mint_amount)
    }

    default fn get_accrual_block_number(&self) -> BlockNumber {
        self._accrual_block_number()
    }

    default fn redeem_underlying(&mut self, redeem_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._redeem(Self::env().caller(), redeem_amount)
    }

    default fn redeem_all(&mut self) -> Result<()> {
        self._accrue_interest()?;
        let caller = Self::env().caller();
        let all_tokens_redeemed = self._balance_of(&caller);
        self._redeem(caller, all_tokens_redeemed)
    }

    default fn borrow(&mut self, borrow_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._borrow(Self::env().caller(), borrow_amount, true)
    }

    #[modifiers(delegated_allowed(borrower, borrow_amount))]
    default fn borrow_for(&mut self, borrower: AccountId, borrow_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._borrow(borrower, borrow_amount, true)?;

        let delegatee = Self::env().caller();
        let delegate_allowance = self._delegate_allowance(&borrower, &delegatee);
        self._approve_delegate(borrower, delegatee, delegate_allowance - borrow_amount)
    }

    #[modifiers(only_flashloan_gateway)]
    default fn borrow_for_flashloan(
        &mut self,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()> {
        self._accrue_interest()?;
        self._borrow(borrower, borrow_amount, false)
    }

    default fn repay_borrow(&mut self, repay_amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._repay_borrow(Self::env().caller(), Self::env().caller(), repay_amount)?;
        Ok(())
    }

    default fn repay_borrow_all(&mut self) -> Result<()> {
        self._accrue_interest()?;
        self._repay_borrow(Self::env().caller(), Self::env().caller(), u128::MAX)?;
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

    #[modifiers(only_flashloan_gateway)]
    default fn transfer_underlying(&mut self, to: AccountId, amount: Balance) -> Result<()> {
        self._accrue_interest()?;
        self._transfer_underlying(to, amount)
    }

    default fn set_controller(&mut self, new_controller: AccountId) -> Result<()> {
        self._assert_manager()?;
        let old = self._controller();
        self._set_controller(new_controller)?;
        self._emit_new_controller_event(old, Some(new_controller));
        Ok(())
    }

    default fn set_reserve_factor_mantissa(
        &mut self,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        self._accrue_interest()?;
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
        self._accrue_interest()?;
        let old = self._rate_model();
        self._set_interest_rate_model(new_interest_rate_model)?;
        self._emit_new_interest_rate_model_event(old, Some(new_interest_rate_model));
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

    default fn set_liquidation_threshold(&mut self, new_liquidation_threshold: u128) -> Result<()> {
        self._assert_manager()?;
        self._set_liquidation_threshold(new_liquidation_threshold)
    }

    default fn approve_delegate(&mut self, delegatee: AccountId, amount: Balance) -> Result<()> {
        self._approve_delegate(Self::env().caller(), delegatee, amount)
    }

    default fn increase_delegate_allowance(
        &mut self,
        delegatee: AccountId,
        amount: Balance,
    ) -> Result<()> {
        let owner = Self::env().caller();
        self._approve_delegate(
            owner,
            delegatee,
            self._delegate_allowance(&owner, &delegatee) + amount,
        )
    }

    default fn decrease_delegate_allowance(
        &mut self,
        delegatee: AccountId,
        amount: Balance,
    ) -> Result<()> {
        let owner = Self::env().caller();
        let delegate_allowance = self._delegate_allowance(&owner, &delegatee);

        if delegate_allowance < amount {
            return Err(Error::InsufficientDelegateAllowance)
        }

        self._approve_delegate(owner, delegatee, delegate_allowance - amount)
    }

    default fn set_use_reserve_as_collateral(&mut self, use_as_collateral: bool) -> Result<()> {
        self._accrue_interest()?;
        let user = Self::env().caller();
        self._validate_set_use_reserve_as_collateral(user, use_as_collateral)?;
        self._set_use_reserve_as_collateral(user, use_as_collateral);
        Ok(())
    }

    default fn set_incentives_controller(
        &mut self,
        incentives_controller: AccountId,
    ) -> Result<()> {
        self._assert_manager()?;
        self._set_incentives_controller(incentives_controller)
    }

    default fn set_manager(&mut self, manager: AccountId) -> Result<()> {
        self._assert_manager()?;
        self._set_manager(manager)?;
        Ok(())
    }

    default fn accept_manager(&mut self) -> Result<()> {
        self._assert_pending_manager()?;
        self._accept_manager()?;
        Ok(())
    }

    default fn underlying(&self) -> Option<AccountId> {
        self._underlying()
    }

    default fn controller(&self) -> Option<AccountId> {
        self._controller()
    }

    default fn manager(&self) -> Option<AccountId> {
        self._manager()
    }

    default fn pending_manager(&self) -> Option<AccountId> {
        self._pending_manager()
    }

    default fn incentives_controller(&self) -> Option<AccountId> {
        self._incentives_controller()
    }

    default fn exchange_rate_stored(&self) -> WrappedU256 {
        WrappedU256::from(self._exchange_rate_stored())
    }

    default fn exchange_rate_current(&mut self) -> Result<WrappedU256> {
        self._accrue_interest()?;
        Ok(self.exchange_rate_stored())
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

    default fn balance_of_underlying(&self, account: AccountId) -> Balance {
        self._balance_of_underlying(account)
    }

    default fn get_account_snapshot(
        &mut self,
        account: AccountId,
    ) -> Result<(Balance, Balance, U256)> {
        self._accrue_interest()?;
        let using_as_collateral = self._using_reserve_as_collateral(account);
        if using_as_collateral.unwrap_or(false) {
            return Ok((
                self._balance_of(&account),
                self._borrow_balance_stored(account),
                self._exchange_rate_stored(),
            ))
        }
        Ok((0, self._balance_of(&account), self._exchange_rate_stored()))
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

    default fn initial_exchange_rate_mantissa(&self) -> WrappedU256 {
        self._initial_exchange_rate_mantissa()
    }

    default fn reserve_factor_mantissa(&self) -> WrappedU256 {
        self._reserve_factor_mantissa()
    }

    default fn liquidation_threshold(&self) -> u128 {
        self._liquidation_threshold()
    }

    default fn delegate_allowance(&self, owner: AccountId, delegatee: AccountId) -> Balance {
        self._delegate_allowance(&owner, &delegatee)
    }

    default fn using_reserve_as_collateral(&self, user: AccountId) -> bool {
        self._using_reserve_as_collateral(user).unwrap_or_default()
    }

    default fn metadata(&self) -> PoolMetaData {
        PoolMetaData {
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            liquidation_threshold: self._liquidation_threshold(),
        }
    }

    default fn status(&self) -> PoolStatus {
        PoolStatus {
            total_supply: self._total_supply(),
            total_borrows: self._total_borrows(),
            exchange_rate: self._exchange_rate_stored(),
        }
    }
}

impl<T: Storage<Data> + Storage<psp22::Data> + Storage<psp22::extensions::metadata::Data>> Internal
    for T
{
    default fn _accrue_interest(&mut self) -> Result<()> {
        self._accrue_interest_at(Self::env().block_number())
    }
    default fn _accrue_interest_at(&mut self, at: BlockNumber) -> Result<()> {
        let accrual_block_number_prior = self._accrual_block_number();
        if accrual_block_number_prior.eq(&at) {
            return Ok(())
        }
        let cash = self._get_cash_prior();
        let borrows = self._total_borrows();
        let reserves = self._total_reserves();
        let idx = self._borrow_index();

        let rate_model = self._rate_model().ok_or(Error::InterestRateModelIsNotSet)?;
        let borrow_rate =
            InterestRateModelRef::get_borrow_rate(&rate_model, cash, borrows, reserves);

        let calculated_interest = calculate_interest(&CalculateInterestInput {
            total_borrows: borrows,
            total_reserves: reserves,
            borrow_index: idx.into(),
            borrow_rate: borrow_rate.into(),
            old_block_number: self._accrual_block_number(),
            new_block_number: at,
            reserve_factor_mantissa: self._reserve_factor_mantissa().into(),
        })?;

        let mut data = self.data::<Data>();
        data.accrual_block_number = at;
        data.borrow_index = calculated_interest.borrow_index.into();
        data.total_borrows = calculated_interest.total_borrows;
        data.total_reserves = calculated_interest.total_reserves;

        self._emit_accrue_interest_event(
            cash,
            calculated_interest.interest_accumulated,
            calculated_interest.borrow_index.into(),
            calculated_interest.total_borrows,
        );
        Ok(())
    }

    default fn _transfer_tokens(
        &mut self,
        spender: AccountId,
        src: AccountId,
        dst: AccountId,
        value: Balance,
        data: Vec<u8>,
    ) -> core::result::Result<(), PSP22Error> {
        let reward_result = self._accrue_reward(src);
        if reward_result.is_err() {
            return Err(PSP22Error::Custom(String::from("AccrueRewardFailed")))
        }

        let reward_result = self._accrue_reward(dst);
        if reward_result.is_err() {
            return Err(PSP22Error::Custom(String::from("AccrueRewardFailed")))
        }

        let accure_result = self.accrue_interest();
        if accure_result.is_err() {
            return Err(PSP22Error::Custom(String::from("AccrueInterestFailed")))
        }

        if src == dst {
            return Err(PSP22Error::Custom(String::from("TransferNotAllowed")))
        }

        let contract_addr = Self::env().account_id();

        // No need to check the error because interest has already updated.
        let account_snapshot_result = self.get_account_snapshot(src);
        let (account_balance, account_borrow_balance, exchange_rate) =
            account_snapshot_result.unwrap();
        let pool_attribute = PoolAttributes {
            pool: Some(contract_addr),
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            liquidation_threshold: self._liquidation_threshold(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
        };

        let controller = self
            ._controller()
            .ok_or(PSP22Error::Custom(String::from("ControllerIsNotSet")))?;
        ControllerRef::transfer_allowed(
            &controller,
            contract_addr,
            src,
            dst,
            value,
            Some(pool_attribute),
        )?;

        if spender == src {
            // copied from PSP22#transfer
            // ref: https://github.com/Brushfam/openbrush-contracts/blob/868ee023727c49296b774327bee25db7b5160c49/contracts/src/token/psp22/psp22.rs#L75-L79
            self._transfer_from_to(src, dst, value, data)?;
        } else {
            // copied from PSP22#transfer_from
            // ref: https://github.com/Brushfam/openbrush-contracts/blob/868ee023727c49296b774327bee25db7b5160c49/contracts/src/token/psp22/psp22.rs#L81-L98
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
        self._accrue_reward(minter)?;
        let contract_addr = Self::env().account_id();

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        ControllerRef::mint_allowed_builder(&controller, contract_addr, minter, mint_amount)
            .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
            .try_invoke()
            .unwrap()
            .unwrap()?;

        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        };

        let exchange_rate = self._exchange_rate_stored(); // NOTE: need exchange_rate calculation before transfer underlying
        let caller = Self::env().caller();

        self._transfer_underlying_from(caller, contract_addr, mint_amount)?;
        let minted_tokens = U256::from(mint_amount)
            .mul(exp_scale())
            .div(exchange_rate)
            .as_u128();

        // Check if it is first deposit.
        let lp_balance = self._balance_of(&minter);
        if lp_balance == 0 {
            self._set_use_reserve_as_collateral(minter, true);
        }

        self._mint_to(minter, minted_tokens)?;
        self._emit_mint_event(minter, mint_amount, minted_tokens);

        Ok(())
    }

    default fn _redeem(&mut self, redeemer: AccountId, redeem_amount: Balance) -> Result<()> {
        self._accrue_reward(redeemer)?;
        if redeem_amount == 0
            || !self
                ._using_reserve_as_collateral(redeemer)
                .unwrap_or_default()
        {
            return Ok(())
        }

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let (_, account_borrow_balance, exchange_rate) = self.get_account_snapshot(redeemer)?;
        let account_balance = self._balance_of(&redeemer);
        let contract_addr = Self::env().account_id();

        let pool_attribute = PoolAttributes {
            pool: Some(contract_addr),
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            liquidation_threshold: self._liquidation_threshold(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
        };
        ControllerRef::redeem_allowed(
            &controller,
            contract_addr,
            redeemer,
            redeem_amount,
            Some(pool_attribute),
        )?;
        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        if self._get_cash_prior() < redeem_amount {
            return Err(Error::RedeemTransferOutNotPossible)
        }

        let lp_balance = self._balance_of(&redeemer);
        if lp_balance == redeem_amount {
            self._set_use_reserve_as_collateral(redeemer, false);
        }

        self._burn_from(redeemer, redeem_amount)?;
        self._transfer_underlying(redeemer, redeem_amount)?;

        self._emit_redeem_event(redeemer, redeem_amount);

        Ok(())
    }

    default fn _borrow(
        &mut self,
        borrower: AccountId,
        borrow_amount: Balance,
        release_underlying: bool,
    ) -> Result<()> {
        self._accrue_reward(borrower)?;

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let contract_addr = Self::env().account_id();
        let caller: ink_primitives::AccountId = Self::env().caller();
        let (account_balance, account_borrow_balance, exchange_rate) =
            self.get_account_snapshot(borrower)?;

        let pool_attribute = PoolAttributes {
            pool: Some(contract_addr),
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
            liquidation_threshold: self._liquidation_threshold(),
        };

        ControllerRef::borrow_allowed(
            &controller,
            contract_addr,
            borrower,
            borrow_amount,
            Some(pool_attribute),
        )?;

        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        };
        if self._get_cash_prior() < borrow_amount {
            return Err(Error::BorrowCashNotAvailable)
        }

        let account_borrows_prev = self._borrow_balance_stored(borrower);
        let account_borrows_new = account_borrows_prev + borrow_amount;
        let total_borrows_new = self._total_borrows() + borrow_amount;

        let borrow_index = self._borrow_index();

        self.data::<Data>().account_borrows.insert(
            &borrower,
            &BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: borrow_index,
            },
        );
        self.data::<Data>().total_borrows = total_borrows_new;

        if release_underlying {
            self._transfer_underlying(caller, borrow_amount)?;
        }

        self._emit_borrow_event(
            borrower,
            borrow_amount,
            account_borrows_new,
            total_borrows_new,
        );

        Ok(())
    }

    default fn _repay_borrow(
        &mut self,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<Balance> {
        self._accrue_reward(borrower)?;
        self._accrue_reward(payer)?;
        let contract_addr = Self::env().account_id();

        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        };

        let account_borrow_prev = self._borrow_balance_stored(borrower);
        let repay_amount_final = if repay_amount > account_borrow_prev {
            account_borrow_prev
        } else {
            repay_amount
        };

        self._transfer_underlying_from(payer, contract_addr, repay_amount_final)?;

        let account_borrows_new = self._borrow_balance_stored(borrower) - repay_amount_final;
        let total_borrows_new = self._total_borrows() - repay_amount_final;

        let borrow_index = self._borrow_index();

        self.data::<Data>().account_borrows.insert(
            &borrower,
            &BorrowSnapshot {
                principal: account_borrows_new,
                interest_index: borrow_index,
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

        Ok(repay_amount_final)
    }

    default fn _liquidate_borrow(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()> {
        self._accrue_reward(liquidator)?;
        self._accrue_reward(borrower)?;

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let contract_addr = Self::env().account_id();
        let (account_balance, account_borrow_balance, exchange_rate) =
            self.get_account_snapshot(borrower)?;
        let pool_attribute = PoolAttributes {
            pool: Some(contract_addr),
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
            liquidation_threshold: self._liquidation_threshold(),
        };

        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }
        if collateral != contract_addr {
            if PoolRef::get_accrual_block_number(&collateral) != current_block_number {
                return Err(Error::AccrualBlockNumberIsNotFresh)
            }
        }
        if liquidator == borrower {
            return Err(Error::LiquidateLiquidatorIsBorrower)
        }
        if repay_amount == 0 {
            return Err(Error::LiquidateCloseAmountIsZero)
        }

        ControllerRef::liquidate_borrow_allowed(
            &controller,
            contract_addr,
            collateral,
            liquidator,
            borrower,
            repay_amount,
            Some(pool_attribute),
        )?;

        let actual_repay_amount = self._repay_borrow(liquidator, borrower, repay_amount)?;
        let pool_borrowed_attributes = Some(PoolAttributesForSeizeCalculation {
            underlying: self._underlying(),
            decimals: self.token_decimals(),
        });
        let seize_tokens = if collateral == contract_addr {
            let pool_collateral_attributes = pool_borrowed_attributes.clone();
            let seize_tokens = ControllerRef::liquidate_calculate_seize_tokens(
                &controller,
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
                &controller,
                contract_addr,
                collateral,
                PoolRef::exchange_rate_stored(&collateral),
                actual_repay_amount,
                pool_borrowed_attributes,
                Some(PoolAttributesForSeizeCalculation {
                    underlying: PoolRef::underlying(&collateral),
                    decimals: PoolRef::token_decimals(&collateral),
                }),
            )?;

            // Check if controller to prevent cross-contract calling (Callee Trapped Error.)
            let seizer_controller: AccountId =
                PoolRef::controller(&collateral).ok_or(Error::ControllerIsNotSet)?;

            if seizer_controller != controller {
                return Err(Error::from(ControllerError::ControllerMismatch))
            }
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

        Ok(())
    }

    default fn _seize(
        &mut self,
        seizer_token: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()> {
        if liquidator == borrower {
            return Err(Error::LiquidateSeizeLiquidatorIsBorrower)
        }
        self._accrue_reward(borrower)?;
        self._accrue_reward(liquidator)?;

        let contract_addr = Self::env().account_id();

        let collateral_enabled = self._using_reserve_as_collateral(borrower).unwrap_or(false);
        if !collateral_enabled {
            return Err(Error::ReserveIsNotEnabledAsCollateral)
        }

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        ControllerRef::seize_allowed(
            &controller,
            contract_addr,
            seizer_token,
            liquidator,
            borrower,
            seize_tokens,
        )?;

        // calculate the new borrower and liquidator token balances
        let exchange_rate = Exp {
            mantissa: WrappedU256::from(self._exchange_rate_stored()),
        };
        let (liquidator_seize_tokens, protocol_seize_amount, _) =
            protocol_seize_amount(exchange_rate, seize_tokens, protocol_seize_share_mantissa());
        let total_reserves_new = self._total_reserves() + protocol_seize_amount;

        // EFFECTS & INTERACTIONS
        self.data::<Data>().total_reserves = total_reserves_new;
        self._burn_from(borrower, seize_tokens)?;
        self._mint_to(liquidator, liquidator_seize_tokens)?;

        self._emit_reserves_added_event(contract_addr, protocol_seize_amount, total_reserves_new);

        Ok(())
    }

    // admin functions
    default fn _set_controller(&mut self, new_controller: AccountId) -> Result<()> {
        self.data::<Data>().controller = Some(new_controller);
        Ok(())
    }

    default fn _set_reserve_factor_mantissa(
        &mut self,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._accrue_interest()?;

        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
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
        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        self.data::<Data>().rate_model = Some(new_interest_rate_model);
        Ok(())
    }

    default fn _add_reserves(&mut self, amount: Balance) -> Result<()> {
        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
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
        let current_block_number = Self::env().block_number();
        if self._accrual_block_number() != current_block_number {
            return Err(Error::AccrualBlockNumberIsNotFresh)
        }

        if self._get_cash_prior().lt(&amount) {
            return Err(Error::ReduceReservesCashNotAvailable)
        }
        if self._total_reserves().lt(&amount) {
            return Err(Error::ReduceReservesCashValidation)
        }
        let total_reserves_new = self._total_reserves().sub(amount);
        self.data::<Data>().total_reserves = total_reserves_new;
        self._transfer_underlying(admin, amount)?;

        // event
        self._emit_reserves_reduced_event(amount, total_reserves_new);
        Ok(())
    }

    default fn _sweep_token(&mut self, asset: AccountId) -> Result<()> {
        let underlying = self._underlying().ok_or(Error::UnderlyingIsNotSet)?;
        if asset == underlying {
            return Err(Error::CannotSweepUnderlyingToken)
        }

        let balance = PSP22Ref::balance_of(&asset, Self::env().account_id());
        PSP22Ref::transfer(&asset, Self::env().caller(), balance, Vec::<u8>::new())?;

        Ok(())
    }

    default fn _set_liquidation_threshold(
        &mut self,
        new_liquidation_threshold: u128,
    ) -> Result<()> {
        let contract_addr = Self::env().account_id();
        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let collateral_factor_result: Option<WrappedU256> =
            ControllerRef::collateral_factor_mantissa(&controller, contract_addr);
        let collateral_factor = collateral_factor_result
            .ok_or(Error::from(ControllerError::InvalidCollateralFactor))?;

        if U256::from(collateral_factor).ge(&U256::from(new_liquidation_threshold).mul(U256::from(
            10_u128.pow(COLLATERAL_FACTOR_MANTISSA_DECIMALS - LIQUIDATION_THRESHOLD_DECIMALS),
        ))) {
            return Err(Error::InvalidLiquidationThreshold)
        }

        self.data::<Data>().liquidation_threshold = new_liquidation_threshold;
        Ok(())
    }

    default fn _approve_delegate(
        &mut self,
        owner: AccountId,
        delegatee: AccountId,
        amount: Balance,
    ) -> Result<()> {
        if owner.is_zero() {
            return Err(Error::ZeroOwnerAddress)
        }
        if delegatee.is_zero() {
            return Err(Error::ZeroDelegateeAddress)
        }

        self.data::<Data>()
            .delegate_allowance
            .insert(&(&owner, &delegatee), &amount);

        self._emit_delegate_approval_event(owner, delegatee, amount);
        Ok(())
    }

    default fn _set_use_reserve_as_collateral(&mut self, user: AccountId, use_as_collateral: bool) {
        let current_using_as_collateral = self
            .data::<Data>()
            .using_reserve_as_collateral
            .get(&user)
            .unwrap_or(false);
        if current_using_as_collateral == use_as_collateral {
            return
        }

        self.data::<Data>()
            .using_reserve_as_collateral
            .insert(&user, &use_as_collateral);

        if use_as_collateral {
            self._emit_reserve_used_as_collateral_enabled_event(user);
        } else {
            self._emit_reserve_used_as_collateral_disabled_event(user);
        }
    }

    default fn _validate_set_use_reserve_as_collateral(
        &mut self,
        user: AccountId,
        use_as_collateral: bool,
    ) -> Result<()> {
        if use_as_collateral || !self._using_reserve_as_collateral(user).unwrap_or_default() {
            return Ok(())
        }

        let (account_balance, account_borrow_balance, exchange_rate) =
            self.get_account_snapshot(user)?;
        if account_balance == 0 {
            return Err(Error::from(PSP22Error::InsufficientBalance))
        }

        let contract_addr = Self::env().account_id();

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let pool_attributes: PoolAttributes = PoolAttributes {
            pool: Some(contract_addr),
            underlying: self._underlying(),
            decimals: self.token_decimals(),
            liquidation_threshold: self._liquidation_threshold(),
            account_balance,
            account_borrow_balance,
            exchange_rate,
            total_borrows: self._total_borrows(),
        };

        ControllerRef::balance_decrease_allowed(
            &controller,
            pool_attributes,
            user,
            account_balance,
        )?;

        Ok(())
    }

    default fn _accrue_reward(&self, user: AccountId) -> Result<()> {
        if let Some(incentives_controller) = self._incentives_controller() {
            let handle_result = IncentivesControllerRef::handle_action(
                &incentives_controller,
                user,
                self._total_supply(),
                self._total_borrows(),
                self._balance_of(&user),
                self._borrow_balance_stored(user),
            );

            if handle_result.is_ok() {
                return Ok(())
            }
            return Err(Error::AccrueRewardFailed)
        }
        Ok(())
    }

    // utilities
    default fn _transfer_underlying_from(
        &self,
        from: AccountId,
        to: AccountId,
        value: Balance,
    ) -> Result<()> {
        let underlying = self._underlying().ok_or(Error::UnderlyingIsNotSet)?;
        PSP22Ref::transfer_from_builder(&underlying, from, to, value, Vec::<u8>::new())
            .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))
            .try_invoke()
            .unwrap()
            .unwrap()
            .map_err(to_psp22_error)
    }

    default fn _transfer_underlying(&self, to: AccountId, value: Balance) -> Result<()> {
        let underlying = self._underlying().ok_or(Error::UnderlyingIsNotSet)?;
        PSP22Ref::transfer(&underlying, to, value, Vec::<u8>::new()).map_err(to_psp22_error)
    }

    default fn _assert_manager(&self) -> Result<()> {
        let manager = self._manager().ok_or(Error::ManagerIsNotSet)?;
        if Self::env().caller() != manager {
            return Err(Error::CallerIsNotManager)
        }

        Ok(())
    }

    default fn _assert_pending_manager(&self) -> Result<()> {
        let pending_manager = self
            ._pending_manager()
            .ok_or(Error::PendingManagerIsNotSet)?;
        if Self::env().caller() != pending_manager {
            return Err(Error::CallerIsNotPendingManager)
        }

        Ok(())
    }

    default fn _set_incentives_controller(
        &mut self,
        incentives_controller: AccountId,
    ) -> Result<()> {
        self.data::<Data>().incentives_controller = Some(incentives_controller);
        Ok(())
    }

    default fn _set_manager(&mut self, manager: AccountId) -> Result<()> {
        self.data::<Data>().pending_manager = Some(manager);
        Ok(())
    }

    default fn _accept_manager(&mut self) -> Result<()> {
        let manager = self._manager().ok_or(Error::ManagerIsNotSet)?;
        let pending_manager = self
            ._pending_manager()
            .ok_or(Error::PendingManagerIsNotSet)?;
        self.data::<Data>().manager = Some(pending_manager);
        self.data::<Data>().pending_manager = None;

        self._emit_manager_updated_event(manager, pending_manager);
        Ok(())
    }

    // view functions
    default fn _underlying(&self) -> Option<AccountId> {
        self.data::<Data>().underlying
    }

    default fn _controller(&self) -> Option<AccountId> {
        self.data::<Data>().controller
    }

    default fn _manager(&self) -> Option<AccountId> {
        self.data::<Data>().manager
    }

    default fn _pending_manager(&self) -> Option<AccountId> {
        self.data::<Data>().pending_manager
    }

    default fn _incentives_controller(&self) -> Option<AccountId> {
        self.data::<Data>().incentives_controller
    }

    default fn _get_cash_prior(&self) -> Balance {
        if let Some(underlying) = self._underlying() {
            let contract_addr = Self::env().account_id();
            return PSP22Ref::balance_of(&underlying, contract_addr)
        }
        0
    }

    default fn _total_borrows(&self) -> Balance {
        self.data::<Data>().total_borrows
    }

    default fn _rate_model(&self) -> Option<AccountId> {
        self.data::<Data>().rate_model
    }

    default fn _borrow_rate_per_msec(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> WrappedU256 {
        if let Some(rate_model) = self._rate_model() {
            return InterestRateModelRef::get_borrow_rate(&rate_model, cash, borrows, reserves)
        }

        WrappedU256::from(U256::zero())
    }

    default fn _supply_rate_per_msec(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: WrappedU256,
    ) -> WrappedU256 {
        if let Some(rate_model) = self._rate_model() {
            return InterestRateModelRef::get_supply_rate(
                &rate_model,
                cash,
                borrows,
                reserves,
                reserve_factor_mantissa,
            )
        }

        WrappedU256::from(U256::zero())
    }

    default fn _accrual_block_number(&self) -> BlockNumber {
        self.data::<Data>().accrual_block_number
    }

    default fn _total_reserves(&self) -> Balance {
        self.data::<Data>().total_reserves
    }

    default fn _borrow_index(&self) -> WrappedU256 {
        self.data::<Data>().borrow_index
    }

    default fn _borrow_balance_stored(&self, account: AccountId) -> Balance {
        let snapshot = self
            .data::<Data>()
            .account_borrows
            .get(&account)
            .unwrap_or_default();

        if snapshot.principal == 0 {
            return 0
        }

        let borrow_index = self._borrow_index();
        let principal_times_index = U256::from(snapshot.principal).mul(U256::from(borrow_index));
        principal_times_index
            .div(U256::from(snapshot.interest_index))
            .as_u128()
    }

    default fn _balance_of_underlying(&self, account: AccountId) -> Balance {
        let exchange_rate = Exp {
            mantissa: self._exchange_rate_stored().into(),
        };
        let pool_token_balance = self._balance_of(&account);
        underlying_balance(exchange_rate, pool_token_balance)
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

    default fn _liquidation_threshold(&self) -> u128 {
        self.data::<Data>().liquidation_threshold
    }

    default fn _delegate_allowance(&self, owner: &AccountId, delegatee: &AccountId) -> Balance {
        self.data::<Data>()
            .delegate_allowance
            .get(&(owner, delegatee))
            .unwrap_or(0)
    }

    default fn _using_reserve_as_collateral(&self, user: AccountId) -> Option<bool> {
        self.data::<Data>().using_reserve_as_collateral.get(&user)
    }

    // event emission
    default fn _emit_mint_event(
        &self,
        _minter: AccountId,
        _mint_amount: Balance,
        _mint_tokens: Balance,
    ) {
    }
    default fn _emit_redeem_event(&self, _redeemer: AccountId, _redeem_amount: Balance) {}
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
        _cash_prior: Balance,
        _interest_accumulated: Balance,
        _new_index: WrappedU256,
        _new_total_borrows: Balance,
    ) {
    }

    default fn _emit_reserves_reduced_event(
        &self,
        _reduce_amount: Balance,
        _total_reserves_new: Balance,
    ) {
    }

    default fn _emit_new_controller_event(&self, _old: Option<AccountId>, _new: Option<AccountId>) {
    }
    default fn _emit_new_interest_rate_model_event(
        &self,
        _old: Option<AccountId>,
        _new: Option<AccountId>,
    ) {
    }
    default fn _emit_new_reserve_factor_event(&self, _old: WrappedU256, _new: WrappedU256) {}
    default fn _emit_delegate_approval_event(
        &self,
        _owner: AccountId,
        _delegatee: AccountId,
        _amount: Balance,
    ) {
    }

    default fn _emit_reserve_used_as_collateral_enabled_event(&self, _user: AccountId) {}
    default fn _emit_reserve_used_as_collateral_disabled_event(&self, _user: AccountId) {}
    default fn _emit_manager_updated_event(&self, _old: AccountId, _new: AccountId) {}
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
            controller::Error::CallerIsNotPendingManager => convert("CallerIsNotPendingManager"),
            controller::Error::InvalidCollateralFactor => convert("InvalidCollateralFactor"),
            controller::Error::UnderlyingIsNotSet => convert("UnderlyingIsNotSet"),
            controller::Error::PoolIsNotSet => convert("PoolIsNotSet"),
            controller::Error::ManagerIsNotSet => convert("ManagerIsNotSet"),
            controller::Error::PendingManagerIsNotSet => convert("PendingManagerIsNotSet"),
            controller::Error::OracleIsNotSet => convert("OracleIsNotSet"),
            controller::Error::BalanceDecreaseNotAllowed => convert("BalanceDecreaseNotAllowed"),
            controller::Error::MarketCountReachedToMaximum => {
                convert("MarketCountReachedToMaximum")
            }
            controller::Error::PoolError => convert("PoolError"),
        }
    }
}
