#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::impls::manager::*;
    use openbrush::traits::{
        Storage,
        ZERO_ADDRESS,
    };

    #[ink(storage)]
    #[derive(Storage)]
    pub struct ManagerContract {
        #[storage_field]
        manager: Data,
    }

    impl Manager for ManagerContract {}

    impl ManagerContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                manager: Data {
                    controller: ZERO_ADDRESS.into(),
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

            let _contract = ManagerContract::new();
        }
    }
}
