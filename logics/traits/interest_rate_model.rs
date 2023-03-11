use openbrush::traits::Balance;

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type InterestRateModelRef = dyn InterestRateModel;

#[openbrush::trait_definition]
pub trait InterestRateModel {
    #[ink(message)]
    fn get_borrow_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> WrappedU256;

    #[ink(message)]
    fn get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: Balance,
    ) -> WrappedU256;
}
