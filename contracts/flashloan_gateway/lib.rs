#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

/// Definition of Flashloan Gateway Contract
#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
    };

    use logics::impls::flashloan_gateway::{
        Data,
        Internal,
        *,
    };
    use openbrush::traits::Storage;

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct FlashloanGatewayContract {
        #[storage_field]
        gateway: Data,
    }

    /// Event: Flashloan is executed.
    #[ink(event)]
    pub struct FlashLoan {
        #[ink(topic)]
        target: AccountId,
        #[ink(topic)]
        initiator: AccountId,
        asset: AccountId,
        amount: Balance,
        premium: Balance,
    }

    impl Internal for FlashloanGatewayContract {
        fn _emit_flashloan_event(
            &self,
            target: AccountId,
            initiator: AccountId,
            asset: AccountId,
            amount: Balance,
            premium: Balance,
        ) {
            self.env().emit_event(FlashLoan {
                target,
                initiator,
                asset,
                amount,
                premium,
            })
        }
    }
    impl FlashloanGateway for FlashloanGatewayContract {}

    impl FlashloanGatewayContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(controller: AccountId) -> Self {
            let mut instance = Self::default();
            instance._initialize(controller);
            instance
        }
    }
}
