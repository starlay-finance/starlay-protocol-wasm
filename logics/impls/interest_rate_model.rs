pub use crate::traits::interest_rate_model::*;
use openbrush::traits::{
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
    fn _get_borrow_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> u128;
    fn _get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserve_factor_mantissa: Balance,
    ) -> u128;
}

impl<T: Storage<Data>> InterestRateModel for T {
    default fn get_borrow_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> u128 {
        self._get_borrow_rate(cash, borrows, reserves)
    }

    default fn get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserve_factor_mantissa: Balance,
    ) -> u128 {
        self._get_supply_rate(cash, borrows, reserve_factor_mantissa)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _get_borrow_rate(&self, _cash: Balance, _borrows: Balance, _reserves: Balance) -> u128 {
        todo!()
    }
    default fn _get_supply_rate(
        &self,
        _cash: Balance,
        _borrows: Balance,
        _reserve_factor_mantissa: Balance,
    ) -> u128 {
        todo!()
    }
}
