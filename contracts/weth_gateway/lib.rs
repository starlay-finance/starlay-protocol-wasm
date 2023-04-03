#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::impls::weth_gateway::{
        Data,
        Internal,
        *,
    };
    use openbrush::{
        contracts::ownable::*,
        traits::Storage,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct WETHGatewayContract {
        #[storage_field]
        gateway: Data,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl Ownable for WETHGatewayContract {}

    impl Internal for WETHGatewayContract {}
    impl WETHGateway for WETHGatewayContract {}

    impl WETHGatewayContract {
        #[ink(constructor)]
        pub fn new(weth: AccountId) -> Self {
            let mut instance = Self::default();
            let caller = Self::env().caller();
            instance._init_with_owner(caller);
            instance.gateway.weth = weth;

            instance
        }
    }
}
