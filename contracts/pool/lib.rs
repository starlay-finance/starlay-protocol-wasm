#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::impls::pool::*;
    use openbrush::{
        contracts::psp22::{
            extensions::metadata,
            psp22,
        },
        traits::Storage,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct PoolContract {
        #[storage_field]
        pool: Data,
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl Pool for PoolContract {}

    impl psp22::PSP22 for PoolContract {}

    impl metadata::PSP22Metadata for PoolContract {}

    impl PoolContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
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
