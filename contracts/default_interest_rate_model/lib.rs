// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of Interest Rate Model Contract
#[openbrush::contract]
pub mod contract {
    use logics::{
        impls::interest_rate_model::*,
        traits::types::WrappedU256,
    };
    use openbrush::traits::Storage;

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Storage)]
    pub struct DefaultInterestRateModelContract {
        #[storage_field]
        model: Data,
    }

    impl InterestRateModel for DefaultInterestRateModelContract {}

    impl DefaultInterestRateModelContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(
            base_rate_per_year: WrappedU256,
            multiplier_per_year_slope_1: WrappedU256,
            multiplier_per_year_slope_2: WrappedU256,
            kink: WrappedU256,
        ) -> Self {
            Self {
                model: Data::new(
                    base_rate_per_year,
                    multiplier_per_year_slope_1,
                    multiplier_per_year_slope_2,
                    kink,
                ),
            }
        }
    }
}
