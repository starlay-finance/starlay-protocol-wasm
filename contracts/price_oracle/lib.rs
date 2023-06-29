#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of PriceOracle Contract
#[openbrush::contract]
pub mod price_oracle {
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
