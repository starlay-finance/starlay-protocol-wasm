#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::{
        impls::interest_rate_model::*,
        traits::types::WrappedU256,
    };
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Storage)]
    pub struct DefaultInterestRateModelContract {
        #[storage_field]
        model: Data,
    }

    impl InterestRateModel for DefaultInterestRateModelContract {}

    impl DefaultInterestRateModelContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                model: Data::new(
                    WrappedU256::from(0),
                    WrappedU256::from(0),
                    WrappedU256::from(0),
                ),
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

            let _contract = DefaultInterestRateModelContract::new();
        }
    }
}
