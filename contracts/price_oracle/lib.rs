#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

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

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{
                self,
                DefaultAccounts,
            },
            DefaultEnvironment,
        };
        use openbrush::traits::AccountId;

        fn default_accounts() -> DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }
        fn set_caller(id: AccountId) {
            test::set_caller::<DefaultEnvironment>(id);
        }

        #[ink::test]
        fn new_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let _contract = PriceOracleContract::new();
        }

        #[ink::test]
        fn set_fixed_price_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let mut contract = PriceOracleContract::new();

            let asset_addr = AccountId::from([0x01; 32]);
            assert!(contract
                .set_fixed_price(asset_addr, PRICE_PRECISION * 101 / 100)
                .is_ok());
            assert_eq!(
                contract.get_price(asset_addr),
                Some(PRICE_PRECISION * 101 / 100)
            )
        }
    }
}
