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

/// Definition of PriceOracle Contract
#[openbrush::contract]
pub mod contract {
    use logics::impls::price_oracle::*;
    use openbrush::traits::Storage;

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Storage)]
    pub struct PriceOracleContract {
        #[storage_field]
        price_oracle: Data,
    }

    impl PriceOracle for PriceOracleContract {}

    impl Default for PriceOracleContract {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PriceOracleContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                price_oracle: Data {
                    fixed_prices: Default::default(),
                },
            }
        }
    }
}
