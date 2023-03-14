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
    multiplier_per_milli_second_slope_1: WrappedU256,
    multiplier_per_milli_second_slope_2: WrappedU256,
    base_rate_per_milli_second: WrappedU256,
    kink: WrappedU256,
}

fn base() -> U256 {
    // 1e18
    U256::from_dec_str("1000000000000000000").unwrap()
}

fn milliseconds_per_year() -> U256 {
    U256::from(60 * 60 * 24 * 365).mul(U256::from(1000))
}

fn u256_from_balance(b: Balance) -> U256 {
    U256::from(b)
}
fn utilization_rate(cash: Balance, borrows: Balance, reserves: Balance) -> U256 {
    let (_cash, _borrows, _reserves) = (
        u256_from_balance(cash),
        u256_from_balance(borrows),
        u256_from_balance(reserves),
    );
    if _borrows.eq(&U256::zero()) {
        return U256::zero()
    }
    _borrows.mul(base()).div(_cash.add(_borrows).sub(_reserves))
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
        let to_seconds_func = |val: WrappedU256| -> WrappedU256 {
            WrappedU256::from(U256::from(val).div(milliseconds_per_year()))
        };
        Self {
            multiplier_per_milli_second_slope_1: to_seconds_func(multiplier_per_year_slope_1),
            multiplier_per_milli_second_slope_2: to_seconds_func(multiplier_per_year_slope_2),
            base_rate_per_milli_second: to_seconds_func(base_rate_per_year),
            kink,
        }
    }

    fn borrow_rate(&self, _cash: Balance, _borrows: Balance, _reserves: Balance) -> WrappedU256 {
        let util = utilization_rate(_cash, _borrows, _reserves);
        let data = self;
        if util.le(&U256::from(data.kink)) {
            let result = util
                .mul(U256::from(data.multiplier_per_milli_second_slope_1))
                .div(base())
                .add(data.base_rate_per_milli_second);
            return WrappedU256::from(result)
        }
        let normal_rate = U256::from(data.kink)
            .mul(U256::from(self.multiplier_per_milli_second_slope_1))
            .div(base())
            .add(U256::from(data.base_rate_per_milli_second));
        let excess_util = util.sub(U256::from(data.kink));
        let excess_rate = excess_util
            .mul(U256::from(data.multiplier_per_milli_second_slope_2))
            .div(base());
        WrappedU256::from(normal_rate.add(excess_rate))
    }

    fn supply_rate(
        &self,
        _cash: Balance,
        _borrows: Balance,
        _reserves: Balance,
        _reserve_factor_mantissa: Balance,
    ) -> WrappedU256 {
        let one_minus_reserve_factor =
            U256::from(base()).sub(u256_from_balance(_reserve_factor_mantissa));
        let borrow_rate = self._get_borrow_rate(_cash, _borrows, _reserves);
        let rate_to_pool = U256::from(borrow_rate)
            .mul(one_minus_reserve_factor)
            .div(base());
        WrappedU256::from(
            utilization_rate(_cash, _borrows, _reserves)
                .mul(rate_to_pool)
                .div(base()),
        )
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
        self.data().borrow_rate(_cash, _borrows, _reserves)
    }
    default fn _get_supply_rate(
        &self,
        _cash: Balance,
        _borrows: Balance,
        _reserves: Balance,
        _reserve_factor_mantissa: Balance,
    ) -> WrappedU256 {
        self.data()
            .supply_rate(_cash, _borrows, _reserves, _reserve_factor_mantissa)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use primitive_types::U256;

    fn mul_base(val: u128) -> U256 {
        U256::from(val).mul(base())
    }

    fn percent(val: u128) -> U256 {
        mul_base(val).div(U256::from(100))
    }
    fn wr(val: U256) -> WrappedU256 {
        WrappedU256::from(val)
    }

    #[test]
    fn test_utilization_rate() {
        // Utilization Rate = Borrows / (Cash + Borrows - Reserves)
        let total_borrow: u32 = 100;
        let total_cash: u32 = 900;
        let reserves: u32 = 0;
        let rate = utilization_rate(
            Balance::from(total_cash),
            Balance::from(total_borrow),
            Balance::from(reserves),
        );
        assert_eq!(rate, percent(10))
    }
    #[test]
    fn test_borrow_rate_utilization_under_kink() {
        let total_borrow: u32 = 100;
        let total_cash: u32 = 900;
        let reserves: u32 = 0;
        let kink = 15;

        struct Case {
            multiplier: u128,
            base_rate: u128,
        }
        let cases = [
            Case {
                multiplier: 100,
                base_rate: 100,
            },
            Case {
                multiplier: 99,
                base_rate: 20,
            },
            Case {
                multiplier: 50,
                base_rate: 50,
            },
        ];
        for case in cases {
            let util = utilization_rate(
                Balance::from(total_cash),
                Balance::from(total_borrow),
                Balance::from(reserves),
            );
            // borrow rate = utilization rate(%) * multiplier + base_rate(%)
            let want = util
                .mul(mul_base(case.multiplier))
                .div(base())
                .add(mul_base(case.base_rate));
            let result = Data::new(
                wr(milliseconds_per_year().mul(mul_base(case.base_rate))),
                wr(milliseconds_per_year().mul(mul_base(case.multiplier))),
                wr(milliseconds_per_year().mul(mul_base(case.multiplier))),
                wr(percent(kink)),
            )
            .borrow_rate(
                Balance::from(total_cash),
                Balance::from(total_borrow),
                Balance::from(reserves),
            );

            assert_eq!(U256::from(result), U256::from(want))
        }
    }

    #[test]
    fn test_borrow_rate_utilization_over_kink() {
        let total_borrow: u32 = 100;
        let total_cash: u32 = 900;
        let reserves: u32 = 0;
        let kink = 9;
        let util = utilization_rate(
            Balance::from(total_cash),
            Balance::from(total_borrow),
            Balance::from(reserves),
        );
        struct Case {
            multiplier: u128,
            jump_multiplier: u128,
            base_rate: u128,
        }
        let cases = [
            Case {
                multiplier: 100,
                jump_multiplier: 110,
                base_rate: 100,
            },
            Case {
                multiplier: 99,
                jump_multiplier: 120,
                base_rate: 20,
            },
            Case {
                multiplier: 50,
                jump_multiplier: 130,
                base_rate: 50,
            },
        ];
        for case in cases {
            // borrow rate = multiplier * kink + jump multiplier * (utilization - kink) + base rate
            let want = percent(kink)
                .mul(mul_base(case.multiplier))
                .div(base())
                .add(mul_base(case.base_rate))
                .add(
                    mul_base(case.jump_multiplier)
                        .mul(util.sub(percent(kink)))
                        .div(base()),
                );

            let result = Data::new(
                wr(milliseconds_per_year().mul(mul_base(case.base_rate))),
                wr(milliseconds_per_year().mul(mul_base(case.multiplier))),
                wr(milliseconds_per_year().mul(mul_base(case.jump_multiplier))),
                wr(percent(kink)),
            )
            .borrow_rate(
                Balance::from(total_cash),
                Balance::from(total_borrow),
                Balance::from(reserves),
            );
            assert_eq!(U256::from(result), U256::from(want))
        }
    }
    #[test]
    fn test_get_supply_rate() {
        // TODO
    }
}
