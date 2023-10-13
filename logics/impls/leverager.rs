// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::ops::{
    // Add,
    Div,
    Mul,
    Sub,
};

use super::{
    controller::{
        calculate_available_borrow_in_base_currency,
        calculate_health_factor_from_balances,
        AccountData,
        ControllerRef,
    },
    pool::PoolRef,
    price_oracle::{
        PriceOracleRef,
        PRICE_PRECISION,
    },
    weth::WETHRef,
};
pub use crate::traits::{
    leverager::*,
    types::WrappedU256,
};
use openbrush::{
    contracts::psp22::PSP22Ref,
    traits::{
        AccountId,
        Balance,
        Storage,
    },
};
use primitive_types::U256;

pub const CLOSE_MAX_LOOPS: u128 = 40;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug, Default)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// AccountId of Controller
    pub controller: Option<AccountId>,
    /// AccountId of Weth
    pub weth: Option<AccountId>,
    /// AccountId of Price Oracle
    pub price_oracle: Option<AccountId>,
    /// AccountId of Manager
    pub manager: Option<AccountId>,
}

pub trait Internal {
    fn _controller(&self) -> Option<AccountId>;

    fn _price_oracle(&self) -> Option<AccountId>;

    fn _weth_address(&self) -> Option<AccountId>;

    fn _manager(&self) -> Option<AccountId>;

    fn _get_available_borrows(&self, account: AccountId) -> Option<AvailableBorrows>;

    fn _loan_to_value(&self, asset: AccountId) -> u128;

    fn _liquidation_threshold(&self, asset: AccountId) -> u128;

    fn _get_health_factor(
        &self,
        account: AccountId,
        asset: AccountId,
        withdraw_amount: Balance,
    ) -> U256;

    fn _withdrawable(&self, account: AccountId, asset: AccountId) -> Option<Withdrawable>;

    fn _withdrawable_amount(&self, account: AccountId, asset: AccountId) -> U256;

    fn _assert_manager(&mut self) -> Result<()>;

    fn _initialize(
        &mut self,
        controller: Option<AccountId>,
        price_oracle: Option<AccountId>,
        weth: Option<AccountId>,
    ) -> Result<()>;

    fn _loop_asset(
        &mut self,
        asset: AccountId,
        amount: Balance,
        borrow_ratio: u128,
        loop_count: u128,
    ) -> Result<()>;

    fn _loop_eth(&mut self, borrow_ratio: u128, loop_count: u128) -> Result<()>;

    fn _loop(
        &mut self,
        asset: AccountId,
        amount: Balance,
        borrow_ratio: u128,
        loop_count: u128,
    ) -> Result<()>;

    fn _close(&mut self, asset: AccountId) -> Result<()>;
}

impl<T: Storage<Data>> Leverager for T {
    default fn controller(&self) -> Option<AccountId> {
        self._controller()
    }

    default fn price_oracle(&self) -> Option<AccountId> {
        self._price_oracle()
    }

    default fn weth_address(&self) -> Option<AccountId> {
        self._weth_address()
    }

    default fn manager(&self) -> Option<AccountId> {
        self._manager()
    }

    default fn get_available_borrows(&self, account: AccountId) -> Option<AvailableBorrows> {
        self._get_available_borrows(account)
    }

    default fn get_health_factor(
        &self,
        account: AccountId,
        asset: AccountId,
        withdraw_amount: Balance,
    ) -> U256 {
        self._get_health_factor(account, asset, withdraw_amount)
    }

    default fn loan_to_value(&self, asset: AccountId) -> u128 {
        self._loan_to_value(asset)
    }

    default fn liquidation_threshold(&self, asset: AccountId) -> u128 {
        self._liquidation_threshold(asset)
    }

    default fn withdrawable(&self, account: AccountId, asset: AccountId) -> Option<Withdrawable> {
        self._withdrawable(account, asset)
    }

    default fn withdrawable_amount(&self, account: AccountId, asset: AccountId) -> U256 {
        self._withdrawable_amount(account, asset)
    }

    default fn close(&mut self, asset: AccountId) -> Result<()> {
        self._close(asset)
    }

    default fn initialize(
        &mut self,
        controller: Option<AccountId>,
        price_oracle: Option<AccountId>,
        weth: Option<AccountId>,
    ) -> Result<()> {
        self._assert_manager()?;
        self._initialize(controller, price_oracle, weth)
    }

    default fn loop_asset(
        &mut self,
        asset: AccountId,
        amount: Balance,
        borrow_ratio: u128,
        loop_count: u128,
    ) -> Result<()> {
        self._loop_asset(asset, amount, borrow_ratio, loop_count)
    }

    default fn loop_eth(&mut self, borrow_ratio: u128, loop_count: u128) -> Result<()> {
        self._loop_eth(borrow_ratio, loop_count)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _assert_manager(&mut self) -> Result<()> {
        let caller = Self::env().caller();

        let manager = self._manager().ok_or(Error::ManagerIsNotSet)?;
        if manager != caller {
            return Err(Error::CallerIsNotManager)
        }
        Ok(())
    }

    default fn _initialize(
        &mut self,
        controller: Option<AccountId>,
        price_oracle: Option<AccountId>,
        weth: Option<AccountId>,
    ) -> Result<()> {
        self.data().controller = controller;
        self.data().price_oracle = price_oracle;
        self.data().weth = weth;
        Ok(())
    }

    default fn _controller(&self) -> Option<AccountId> {
        self.data().controller
    }

    default fn _price_oracle(&self) -> Option<AccountId> {
        self.data().price_oracle
    }

    default fn _weth_address(&self) -> Option<AccountId> {
        self.data().weth
    }

    default fn _manager(&self) -> Option<AccountId> {
        self.data().manager
    }

    default fn _get_available_borrows(&self, account: AccountId) -> Option<AvailableBorrows> {
        if let Some(controller) = self._controller() {
            let account_data_result =
                ControllerRef::calculate_user_account_data(&controller, account, None);

            if account_data_result.is_err() {
                return None
            }

            let account_data: AccountData = account_data_result.unwrap();
            let available_borrow_in_base_currency = calculate_available_borrow_in_base_currency(
                account_data.total_collateral_in_base_currency,
                account_data.total_debt_in_base_currency,
                account_data.avg_ltv,
            );

            let price_eth: u128 = if let Some(price_oracle) = self._price_oracle() {
                if let Some(weth) = self._weth_address() {
                    let price = PriceOracleRef::get_price(&price_oracle, weth);
                    price.unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };

            return Some(AvailableBorrows {
                total_collateral_in_base_currency: account_data.total_collateral_in_base_currency,
                available_borrow_in_base_currency,
                health_factor: account_data.health_factor,
                ltv: account_data.avg_ltv,
                price_eth,
            })
        }
        None
    }

    default fn _get_health_factor(
        &self,
        account: AccountId,
        asset: AccountId,
        withdraw_amount: Balance,
    ) -> U256 {
        if let Some(controller) = self._controller() {
            let liquidation_threshold = self._liquidation_threshold(asset);

            let account_data_result =
                ControllerRef::calculate_user_account_data(&controller, account, None);

            if account_data_result.is_err() {
                return U256::from(0)
            }

            let account_data: AccountData = account_data_result.unwrap();

            if let Some(price_oracle) = self._price_oracle() {
                if let Some(price_asset) = PriceOracleRef::get_price(&price_oracle, asset) {
                    let withdraw_amount_in_base_currency = U256::from(price_asset)
                        .mul(U256::from(withdraw_amount))
                        .div(U256::from(PRICE_PRECISION));

                    let total_collateral_after = if account_data.total_collateral_in_base_currency
                        > withdraw_amount_in_base_currency
                    {
                        account_data
                            .total_collateral_in_base_currency
                            .sub(withdraw_amount_in_base_currency)
                    } else {
                        U256::from(0)
                    };

                    let factor = account_data
                        .avg_liquidation_threshold
                        .mul(account_data.total_collateral_in_base_currency)
                        .sub(
                            U256::from(liquidation_threshold).mul(withdraw_amount_in_base_currency),
                        );

                    let liquidation_threshold_after =
                        if !total_collateral_after.is_zero() || factor >= U256::from(0) {
                            factor.div(total_collateral_after)
                        } else {
                            U256::from(0)
                        };

                    return calculate_health_factor_from_balances(
                        total_collateral_after,
                        account_data.total_debt_in_base_currency,
                        liquidation_threshold_after,
                    )
                }
            }

            return U256::from(0)
        }
        U256::from(0)
    }

    default fn _withdrawable(&self, account: AccountId, asset: AccountId) -> Option<Withdrawable> {
        if let Some(controller) = self._controller() {
            let liquidation_threshold = self._liquidation_threshold(asset);

            let account_data_result =
                ControllerRef::calculate_user_account_data(&controller, account, None);

            if account_data_result.is_err() {
                return None
            }

            let account_data: AccountData = account_data_result.unwrap();

            let afford_in_base_currency = account_data
                .total_collateral_in_base_currency
                .mul(account_data.avg_liquidation_threshold)
                .sub(
                    account_data
                        .total_debt_in_base_currency
                        .mul(U256::from(10000)),
                );

            let withdrawable_collateral_in_base_currency =
                afford_in_base_currency.div(U256::from(liquidation_threshold));

            if let Some(price_oracle) = self._price_oracle() {
                if let Some(price_asset) = PriceOracleRef::get_price(&price_oracle, asset) {
                    let withdrawable_collateral = withdrawable_collateral_in_base_currency
                        .mul(U256::from(PRICE_PRECISION))
                        .div(U256::from(price_asset));
                    let mut withdraw_amount = withdrawable_collateral;

                    let mut health_factor =
                        self._get_health_factor(account, asset, withdraw_amount.as_u128());

                    // 1.01
                    let health_factor_limit = U256::from(101 * (10_u128.pow(18)) / 100);
                    while health_factor <= health_factor_limit {
                        // Decrease withdraw amount
                        withdraw_amount = withdraw_amount.mul(U256::from(95)).div(U256::from(100));
                        health_factor =
                            self._get_health_factor(account, asset, withdraw_amount.as_u128());
                    }

                    return Some(Withdrawable {
                        total_collateral_in_base_currency: account_data
                            .total_collateral_in_base_currency,
                        total_debt_in_base_currency: account_data.total_debt_in_base_currency,
                        current_liquidation_threshold: account_data.avg_liquidation_threshold,
                        afford_in_base_currency,
                        withdrawable_collateral_in_base_currency,
                        withdrawable_collateral,
                        withdraw_amount,
                    })
                }
            }
        }
        None
    }

    default fn _withdrawable_amount(&self, account: AccountId, asset: AccountId) -> U256 {
        if let Some(withdrwable) = self._withdrawable(account, asset) {
            return withdrwable.withdraw_amount
        }
        U256::from(0)
    }

    default fn _loan_to_value(&self, asset: AccountId) -> u128 {
        if let Some(controller) = self._controller() {
            if let Some(pool) = ControllerRef::market_of_underlying(&controller, asset) {
                let collateral_factor_result: Option<WrappedU256> =
                    ControllerRef::collateral_factor_mantissa(&controller, pool);
                if let Some(collateral_factor) = collateral_factor_result {
                    // Convert collateral factor into percent
                    return U256::from(collateral_factor)
                        .mul(U256::from(10000))
                        .div(PRICE_PRECISION)
                        .as_u128()
                }
                return 0
            }
            return 0
        }
        0
    }

    default fn _liquidation_threshold(&self, asset: AccountId) -> u128 {
        if let Some(controller) = self._controller() {
            if let Some(pool) = ControllerRef::market_of_underlying(&controller, asset) {
                let liquidation_threshold = PoolRef::liquidation_threshold(&pool);
                return liquidation_threshold
            }
            return 0
        }
        0
    }

    default fn _loop_asset(
        &mut self,
        asset: AccountId,
        amount: Balance,
        borrow_ratio: u128,
        loop_count: u128,
    ) -> Result<()> {
        let _ltv = self._loan_to_value(asset);
        if borrow_ratio <= 0 || borrow_ratio > _ltv {
            return Err(Error::InappropriateBorrowRate)
        }
        if loop_count < 2 || loop_count > 40 {
            return Err(Error::InappropriateLoopCount)
        }

        let caller = Self::env().caller();
        let contract_addr = Self::env().account_id();
        PSP22Ref::transfer_from(&asset, caller, contract_addr, amount, Default::default())?;

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let pool = ControllerRef::market_of_underlying(&controller, asset)
            .ok_or(Error::MarketNotListed)?;
        PSP22Ref::approve(&asset, pool, u128::MAX)?;
        self._loop(asset, amount, borrow_ratio, loop_count)
    }

    default fn _loop_eth(&mut self, borrow_ratio: u128, loop_count: u128) -> Result<()> {
        let weth = self._weth_address().ok_or(Error::WETHIsNotSet)?;
        let _ltv = self._loan_to_value(weth);

        if borrow_ratio <= 0 || borrow_ratio > _ltv {
            return Err(Error::InappropriateBorrowRate)
        }
        if loop_count < 2 || loop_count > 40 {
            return Err(Error::InappropriateLoopCount)
        }

        let deposit_value = Self::env().transferred_value();

        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let pool =
            ControllerRef::market_of_underlying(&controller, weth).ok_or(Error::MarketNotListed)?;
        WETHRef::deposit_builder(&weth)
            .transferred_value(deposit_value)
            .invoke()?;
        WETHRef::approve(&weth, pool, u128::MAX)?;
        self._loop(weth, deposit_value, borrow_ratio, loop_count)
    }

    default fn _loop(
        &mut self,
        asset: AccountId,
        amount: Balance,
        borrow_ratio: u128,
        loop_count: u128,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let pool = ControllerRef::market_of_underlying(&controller, asset)
            .ok_or(Error::MarketNotListed)?;
        let mut next_deposit_amount = amount;
        for _i in 0..loop_count {
            PoolRef::mint_to_builder(&pool, caller, next_deposit_amount)
                .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
                .try_invoke()
                .unwrap()
                .unwrap()?;

            next_deposit_amount = (next_deposit_amount * borrow_ratio) / 10000;

            if next_deposit_amount == 0 {
                break
            }

            PoolRef::borrow_for_builder(&pool, caller, next_deposit_amount)
                .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
                .try_invoke()
                .unwrap()
                .unwrap()?;
        }
        if next_deposit_amount != 0 {
            PoolRef::mint_to_builder(&pool, caller, next_deposit_amount)
                .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
                .try_invoke()
                .unwrap()
                .unwrap()?;
        }
        Ok(())
    }

    default fn _close(&mut self, asset: AccountId) -> Result<()> {
        let caller = Self::env().caller();
        let contract_addr = Self::env().account_id();
        let controller = self._controller().ok_or(Error::ControllerIsNotSet)?;
        let pool = ControllerRef::market_of_underlying(&controller, asset)
            .ok_or(Error::MarketNotListed)?;
        PSP22Ref::approve(&asset, pool, u128::MAX)?;

        let mut withdraw_amount = self._withdrawable_amount(caller, asset).as_u128();
        let mut repay_amount = PoolRef::borrow_balance_current(&pool, caller)?;
        let mut loop_remains = CLOSE_MAX_LOOPS;

        while loop_remains > 0 || withdraw_amount > 0 {
            if withdraw_amount > repay_amount {
                withdraw_amount = repay_amount;

                PoolRef::transfer_from(
                    &pool,
                    caller,
                    contract_addr,
                    withdraw_amount,
                    Default::default(),
                )?;
                PoolRef::redeem(&pool, withdraw_amount)?;
                PoolRef::repay_borrow_behalf(&pool, caller, withdraw_amount)?;
                break
            } else {
                PoolRef::transfer_from(
                    &pool,
                    caller,
                    contract_addr,
                    withdraw_amount,
                    Default::default(),
                )?;
                PoolRef::redeem(&pool, withdraw_amount)?;
                PoolRef::repay_borrow_behalf(&pool, caller, withdraw_amount)?;

                withdraw_amount = self._withdrawable_amount(caller, asset).as_u128();
                repay_amount = PoolRef::borrow_balance_current(&pool, caller)?;
                loop_remains = loop_remains - 1;
            }
        }

        Ok(())
    }
}
