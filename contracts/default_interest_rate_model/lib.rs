#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of Interest Rate Model Contract
#[openbrush::contract]
pub mod default_interest_rate_model {
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
