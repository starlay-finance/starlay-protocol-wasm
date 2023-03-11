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
    // TODO
}

fn base() -> U256 {
    U256::from_dec_str("1000000000000000000").unwrap()
}

fn seconds_per_year() -> U256 {
    U256::from(60 * 60 * 24 * 365)
}

pub trait Internal {
    fn _get_borrow_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> WrappedU256;
    fn _get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserve_factor_mantissa: Balance,
    ) -> WrappedU256;
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
        reserve_factor_mantissa: Balance,
    ) -> WrappedU256 {
        self._get_supply_rate(cash, borrows, reserve_factor_mantissa)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _get_borrow_rate(
        &self,
        _cash: Balance,
        _borrows: Balance,
        _reserves: Balance,
    ) -> WrappedU256 {
        todo!()
    }
    default fn _get_supply_rate(
        &self,
        _cash: Balance,
        _borrows: Balance,
        _reserve_factor_mantissa: Balance,
    ) -> WrappedU256 {
        todo!()
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
