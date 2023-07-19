// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
