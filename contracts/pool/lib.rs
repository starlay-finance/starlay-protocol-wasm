#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::impls::pool::*;
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Storage)]
    pub struct PoolContract {
        #[storage_field]
        pool: Data,
    }

    impl Pool for PoolContract {}

    impl PoolContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { pool: Data {} }
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

            let _contract = PoolContract::new();
        }
    }
}
