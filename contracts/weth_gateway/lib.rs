#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "e2e-tests"))]
pub mod e2e_tests;

/// Definition of WETH Gateway Contract
#[openbrush::contract]
pub mod weth_gateway {
    use ink::env::{
        emit_event,
        DefaultEnvironment,
    };

    use logics::impls::weth_gateway::{
        Data,
        Internal,
        *,
    };
    use openbrush::{
        contracts::ownable::*,
        traits::Storage,
    };

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct WETHGatewayContract {
        #[storage_field]
        gateway: Data,
        #[storage_field]
        ownable: ownable::Data,
    }

    /// Event: DepositETH is executed.
    #[ink(event)]
    pub struct DepositEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub from: AccountId,
        pub value: Balance,
    }

    /// Event: WithdrawEth is executed.
    #[ink(event)]
    pub struct WithdrawEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub to: AccountId,
        pub value: Balance,
    }

    /// Event: BorrowEth is executed.
    #[ink(event)]
    pub struct BorrowEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub to: AccountId,
        pub value: Balance,
    }

    /// Event: RepayEth is executed.
    #[ink(event)]
    pub struct RepayEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub from: AccountId,
        pub value: Balance,
    }

    impl Ownable for WETHGatewayContract {}

    impl Internal for WETHGatewayContract {
        fn _emit_deposit_eth_event_(&self, pool: AccountId, from: AccountId, value: Balance) {
            emit_event::<DefaultEnvironment, _>(DepositEth { pool, from, value });
        }

        fn _emit_withdraw_eth_event_(&self, pool: AccountId, to: AccountId, value: Balance) {
            emit_event::<DefaultEnvironment, _>(WithdrawEth { pool, to, value });
        }

        fn _emit_borrow_eth_event_(&self, pool: AccountId, to: AccountId, value: Balance) {
            emit_event::<DefaultEnvironment, _>(BorrowEth { pool, to, value });
        }

        fn _emit_repay_eth_event_(&self, pool: AccountId, from: AccountId, value: Balance) {
            emit_event::<DefaultEnvironment, _>(RepayEth { pool, from, value });
        }
    }
    impl WETHGateway for WETHGatewayContract {}

    impl WETHGatewayContract {
        /// Generate this contract
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
