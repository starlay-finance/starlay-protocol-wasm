#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::impls::flashloan_receiver::{
        Data,
        Internal,
        *,
    };

    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct FlashloanReceiverContract {
        #[storage_field]
        receiver: Data,
    }

    impl Internal for FlashloanReceiverContract {}
    impl FlashloanReceiver for FlashloanReceiverContract {}

    impl FlashloanReceiverContract {
        #[ink(constructor)]
        pub fn new(flashloan_gateway: AccountId) -> Self {
            let mut _instance = Self::default();
            _instance._initialize(flashloan_gateway);
            _instance
        }
    }
}
