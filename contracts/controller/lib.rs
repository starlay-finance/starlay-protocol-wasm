#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::impls::controller::*;
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Storage)]
    pub struct ControllerContract {
        #[storage_field]
        controller: Data,
    }

    impl Controller for ControllerContract {}

    impl ControllerContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                controller: Data {},
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

            let _contract = ControllerContract::new();
        }
    }
}
