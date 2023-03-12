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
                controller: Data {
                    markets: Default::default(),
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

            let contract = ControllerContract::new();
            assert_eq!(contract.markets(), []);
        }

        #[ink::test]
        fn support_market_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let mut contract = ControllerContract::new();
            let p1 = AccountId::from([0x01; 32]);
            let p2 = AccountId::from([0x02; 32]);
            assert!(contract.support_market(p1).is_ok());
            assert!(contract.support_market(p2).is_ok());
            assert_eq!(contract.markets(), [p1, p2]);
        }
    }
}
