use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};

use crate::traits::types::WrappedU256;
pub use crate::traits::{
    interest_rate_model::*,
    types,
};
use openbrush::traits::{
    Balance,
    Storage,
};
use primitive_types::U256;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    multiplier_per_second_slope_1: WrappedU256,
    multiplier_per_second_slope_2: WrappedU256,
    base_rate_per_second: WrappedU256,
    kink: WrappedU256,
}

fn base() -> U256 {
    U256::from_dec_str("1000000000000000000").unwrap()
}

fn seconds_per_year() -> U256 {
    U256::from(60 * 60 * 24 * 365)
}

fn u256_from_balance(b: Balance) -> U256 {
    U256::from(b.to_be())
}

pub trait Internal {
    fn _get_borrow_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> WrappedU256;
    fn _get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: Balance,
    ) -> WrappedU256;
}

impl Data {
    pub fn new(
        base_rate_per_year: WrappedU256,
        multiplier_per_year_slope_1: WrappedU256,
        multiplier_per_year_slope_2: WrappedU256,
        kink: WrappedU256,
    ) -> Self {
        Self {
            multiplier_per_second_slope_1: WrappedU256::from(
                U256::from(multiplier_per_year_slope_1).div(seconds_per_year()),
            ),
            multiplier_per_second_slope_2: WrappedU256::from(
                U256::from(multiplier_per_year_slope_2).div(seconds_per_year()),
            ),
            base_rate_per_second: WrappedU256::from(
                U256::from(base_rate_per_year).div(seconds_per_year()),
            ),
            kink,
        }
    }

    fn utilization_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> U256 {
        let _cash = u256_from_balance(cash);
        let _borrows = u256_from_balance(borrows);
        let _reserves = u256_from_balance(reserves);
        if _borrows.eq(&U256::zero()) {
            return U256::zero()
        }
        _borrows
            .mul(base())
            .div((_cash.add(_borrows).sub(_reserves)))
    }
}

impl<T: Storage<Data>> InterestRateModel for T {
    default fn get_borrow_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> WrappedU256 {
        self._get_borrow_rate(cash, borrows, reserves)
    }

    default fn get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: Balance,
    ) -> WrappedU256 {
        self._get_supply_rate(cash, borrows, reserves, reserve_factor_mantissa)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _get_borrow_rate(
        &self,
        _cash: Balance,
        _borrows: Balance,
        _reserves: Balance,
    ) -> WrappedU256 {
        let util = self.data().utilization_rate(_cash, _borrows, _reserves);
        let data = self.data();
        let normal_rate = util
            .mul(U256::from(data.multiplier_per_second_slope_1))
            .div(base())
            .add(data.base_rate_per_second);
        if util.le(&U256::from(data.kink)) {
            return WrappedU256::from(normal_rate)
        }
        let excess_util = util.sub(data.kink);
        let excess_rate = excess_util
            .mul(U256::from(data.multiplier_per_second_slope_2))
            .add(data.base_rate_per_second);
        WrappedU256::from(excess_rate)
    }
    default fn _get_supply_rate(
        &self,
        _cash: Balance,
        _borrows: Balance,
        _reserves: Balance,
        _reserve_factor_mantissa: Balance,
    ) -> WrappedU256 {
        let data = self.data();
        let one_minus_reserve_factor =
            U256::from(base()).sub(u256_from_balance(_reserve_factor_mantissa));
        let borrow_rate = self._get_borrow_rate(_cash, _borrows, _reserves);
        let rate_to_pool = U256::from(borrow_rate)
            .mul(one_minus_reserve_factor)
            .div(base());
        WrappedU256::from(
            data.utilization_rate(_cash, _borrows, _reserves)
                .mul(rate_to_pool)
                .div(base()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitive_types::U256;
    #[test]
    fn test_base() {
        assert_eq!(base(), U256::from_dec_str("1000000000000000000").unwrap())
    }
}
