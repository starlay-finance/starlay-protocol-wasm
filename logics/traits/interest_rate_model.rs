use openbrush::traits::Balance;

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type InterestRateModelRef = dyn InterestRateModel;

/// Trait defines the interface for interest rate model
#[openbrush::trait_definition]
pub trait InterestRateModel {
    /// Calculates the current borrow interest rate per milliseconds
    #[ink(message)]
    fn get_borrow_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> WrappedU256;

    /// Calculates the current supply interest rate per milliseconds
    #[ink(message)]
    fn get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: WrappedU256,
    ) -> WrappedU256;
}
