// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

/// Definition of WETH Gateway Contract
#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
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
        pool: AccountId,
        #[ink(topic)]
        from: AccountId,
        value: Balance,
    }

    /// Event: WithdrawEth is executed.
    #[ink(event)]
    pub struct WithdrawEth {
        #[ink(topic)]
        pool: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    /// Event: BorrowEth is executed.
    #[ink(event)]
    pub struct BorrowEth {
        #[ink(topic)]
        pool: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    /// Event: RepayEth is executed.
    #[ink(event)]
    pub struct RepayEth {
        #[ink(topic)]
        pool: AccountId,
        #[ink(topic)]
        from: AccountId,
        value: Balance,
    }

    impl Ownable for WETHGatewayContract {}

    impl Internal for WETHGatewayContract {
        fn _emit_deposit_eth_event_(&self, pool: AccountId, from: AccountId, value: Balance) {
            self.env().emit_event(DepositEth { pool, from, value });
        }

        fn _emit_withdraw_eth_event_(&self, pool: AccountId, to: AccountId, value: Balance) {
            self.env().emit_event(WithdrawEth { pool, to, value });
        }

        fn _emit_borrow_eth_event_(&self, pool: AccountId, to: AccountId, value: Balance) {
            self.env().emit_event(BorrowEth { pool, to, value });
        }

        fn _emit_repay_eth_event_(&self, pool: AccountId, from: AccountId, value: Balance) {
            self.env().emit_event(RepayEth { pool, from, value });
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
