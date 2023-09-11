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
    types,
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

pub const LEVERAGE_CODE: u128 = 10;
pub const CLOSE_MAX_LOOPS: u128 = 40;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug, Default)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub controller: Option<AccountId>,
    pub weth: Option<AccountId>,
    pub price_oracle: Option<AccountId>,
    pub manager: Option<AccountId>,
}

pub trait Internal {
    fn _controller(&self) -> Option<AccountId>;

    fn _price_oracle(&self) -> Option<AccountId>;

    fn _weth_address(&self) -> Option<AccountId>;

    fn _manager(&self) -> Option<AccountId>;

    fn _get_available_borrows(&self, account: AccountId) -> AvailableBorrows;

    fn _loan_to_value(&self, asset: AccountId) -> U256;

    fn _liquidation_threshold(&self, asset: AccountId) -> u128;

    fn _get_health_factor(
        &self,
        account: AccountId,
        asset: AccountId,
        withdraw_amount: Balance,
    ) -> U256;

    fn _withdrawable(&self, account: AccountId, asset: AccountId) -> Withdrawable;

    fn _withdrawable_amount(&self, account: AccountId, asset: AccountId) -> Balance;

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
        interest_rate_mode: U256,
        borrow_ratio: U256,
        loop_count: u128,
    ) -> Result<()>;

    fn _loop_eth(
        &mut self,
        interest_rate_mode: U256,
        borrow_ratio: U256,
        loop_count: u128,
    ) -> Result<()>;

    fn _loop(
        &mut self,
        asset: AccountId,
        amount: Balance,
        interest_rate_mode: U256,
        borrow_ratio: U256,
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

    default fn get_available_borrows(&self, account: AccountId) -> AvailableBorrows {
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

    default fn loan_to_value(&self, asset: AccountId) -> U256 {
        self._loan_to_value(asset)
    }

    default fn liquidation_threshold(&self, asset: AccountId) -> u128 {
        self._liquidation_threshold(asset)
    }

    default fn withdrawable(&self, account: AccountId, asset: AccountId) -> Withdrawable {
        self._withdrawable(account, asset)
    }

    default fn withdrawable_amount(&self, account: AccountId, asset: AccountId) -> Balance {
        self.withdrawable_amount(account, asset)
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
        interest_rate_mode: U256,
        borrow_ratio: U256,
        loop_count: u128,
    ) -> Result<()> {
        self._loop_asset(asset, amount, interest_rate_mode, borrow_ratio, loop_count)
    }

    default fn loop_eth(
        &mut self,
        interest_rate_mode: U256,
        borrow_ratio: U256,
        loop_count: u128,
    ) -> Result<()> {
        self._loop_eth(interest_rate_mode, borrow_ratio, loop_count)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _assert_manager(&mut self) -> Result<()> {
        let caller = Self::env().caller();

        if let Some(manager) = self._manager() {
            if manager != caller {
                return Err(Error::CallerIsNotManager)
            }
            return Ok(())
        }
        Err(Error::ManagerIsNotSet)
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

    default fn _get_available_borrows(&self, account: AccountId) -> AvailableBorrows {
        if let Some(controller) = self._controller() {
            let account_data_result =
                ControllerRef::calculate_user_account_data(&controller, account, None);

            if account_data_result.is_err() {
                return Default::default()
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

            return AvailableBorrows {
                total_collateral_in_base_currency: account_data.total_collateral_in_base_currency,
                available_borrow_in_base_currency,
                health_factor: account_data.health_factor,
                ltv: account_data.avg_ltv,
                price_eth,
            }
        }
        Default::default()
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

    default fn _withdrawable(&self, _account: AccountId, _asset: AccountId) -> Withdrawable {
        Default::default()
    }

    default fn _withdrawable_amount(&self, _account: AccountId, _asset: AccountId) -> Balance {
        0
    }

    default fn _loan_to_value(&self, _asset: AccountId) -> U256 {
        U256::from(0)
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
        interest_rate_mode: U256,
        borrow_ratio: U256,
        loop_count: u128,
    ) -> Result<()> {
        let _ltv = self._loan_to_value(asset);
        if borrow_ratio <= U256::from(0) || borrow_ratio > _ltv {
            return Err(Error::InappropriateBorrowRate)
        }
        if loop_count < 2 || loop_count > 40 {
            return Err(Error::InappropriateLoopCount)
        }

        let caller = Self::env().caller();
        let contract_addr = Self::env().account_id();
        PSP22Ref::transfer_from(&asset, caller, contract_addr, amount, Default::default())?;

        if let Some(controller) = self._controller() {
            if let Some(pool) = ControllerRef::market_of_underlying(&controller, asset) {
                PSP22Ref::approve(&asset, pool, u128::MAX)?;
                return self._loop(asset, amount, interest_rate_mode, borrow_ratio, loop_count)
            }
            return Err(Error::MarketNotListed)
        }
        return Err(Error::ControllerIsNotSet)
    }

    default fn _loop_eth(
        &mut self,
        interest_rate_mode: U256,
        borrow_ratio: U256,
        loop_count: u128,
    ) -> Result<()> {
        if let Some(weth) = self._weth_address() {
            let _ltv = self._loan_to_value(weth);

            if borrow_ratio <= U256::from(0) || borrow_ratio > _ltv {
                return Err(Error::InappropriateBorrowRate)
            }
            if loop_count < 2 || loop_count > 40 {
                return Err(Error::InappropriateLoopCount)
            }

            let deposit_value = Self::env().transferred_value();

            if let Some(controller) = self._controller() {
                if let Some(pool) = ControllerRef::market_of_underlying(&controller, weth) {
                    WETHRef::approve(&weth, pool, u128::MAX)?;
                    WETHRef::deposit_builder(&weth)
                        .transferred_value(deposit_value)
                        .invoke()?;
                    return self._loop(
                        weth,
                        deposit_value,
                        interest_rate_mode,
                        borrow_ratio,
                        loop_count,
                    )
                }
                return Err(Error::MarketNotListed)
            }
        }
        return Err(Error::WETHIsNotSet)
    }

    default fn _loop(
        &mut self,
        _asset: AccountId,
        _amount: Balance,
        _interest_rate_mode: U256,
        _borrow_ratio: U256,
        _loop_count: u128,
    ) -> Result<()> {
        Ok(())
    }

    default fn _close(&mut self, _asset: AccountId) -> Result<()> {
        Ok(())
    }
}
