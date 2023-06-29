#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of Controller Contract
#[openbrush::contract]
pub mod controller {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use logics::impls::controller::{
        Internal,
        *,
    };
    use openbrush::traits::Storage;

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct ControllerContract {
        #[storage_field]
        controller: Data,
    }

    /// Event: Controller starts to support Pool
    #[ink(event)]
    pub struct MarketListed {
        pub pool: AccountId,
    }

    impl Controller for ControllerContract {}

    impl ControllerContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(manager: AccountId) -> Self {
            let mut instance = Self::default();
            instance.controller.manager = manager;
            instance
        }
    }

    impl Internal for ControllerContract {
        fn _emit_market_listed_event(&self, pool: AccountId) {
            self.env().emit_event(MarketListed { pool });
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            build_message,
            subxt::ext::sp_runtime::AccountId32,
            AccountKeyring,
        };
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn test_flipper(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor =
                ControllerContractRef::new(ink_e2e::account_id(AccountKeyring::Alice));
            let controller_id = client
                .instantiate("controller", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
            assert!(controller_id != AccountId::from([0x0; 32]));
            Ok(())
        }
    }
}
