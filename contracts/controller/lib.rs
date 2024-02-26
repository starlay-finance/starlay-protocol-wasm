// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of Controller Contract
#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use logics::{
        impls::controller::{
            Internal,
            *,
        },
        traits::types::WrappedU256,
    };
    use openbrush::traits::{
        Storage,
        String,
    };

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

    /// Event: Controller Manager changed
    #[ink(event)]
    pub struct ManagerAddressUpdated {
        #[ink(topic)]
        pub old: AccountId,
        #[ink(topic)]
        pub new: AccountId,
    }

    #[ink(event)]
    pub struct NewCollateralFactor {
        #[ink(topic)]
        pub pool: AccountId,
        pub old: WrappedU256,
        pub new: WrappedU256,
    }

    #[ink(event)]
    pub struct PoolActionPaused {
        pub pool: AccountId,
        pub action: String,
        pub paused: bool,
    }

    #[ink(event)]
    pub struct ActionPaused {
        pub action: String,
        pub paused: bool,
    }

    #[ink(event)]
    pub struct NewPriceOracle {
        pub old: Option<AccountId>,
        pub new: Option<AccountId>,
    }

    #[ink(event)]
    pub struct NewFlashloanGateway {
        pub old: Option<AccountId>,
        pub new: Option<AccountId>,
    }

    #[ink(event)]
    pub struct NewCloseFactor {
        pub old: WrappedU256,
        pub new: WrappedU256,
    }

    #[ink(event)]
    pub struct NewBorrowCap {
        pub pool: AccountId,
        pub new: Balance,
    }

    #[ink(event)]
    pub struct NewLiquidationIncentive {
        pub old: WrappedU256,
        pub new: WrappedU256,
    }

    impl Controller for ControllerContract {}

    impl ControllerContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(manager: AccountId) -> Self {
            let mut instance = Self::default();
            instance.controller.manager = Some(manager);
            instance
        }
    }

    impl Internal for ControllerContract {
        fn _emit_market_listed_event(&self, pool: AccountId) {
            self.env().emit_event(MarketListed { pool });
        }

        fn _emit_manager_updated_event(&self, old: AccountId, new: AccountId) {
            self.env().emit_event(ManagerAddressUpdated { old, new })
        }

        fn _emit_new_collateral_factor_event(
            &self,
            pool: AccountId,
            old: WrappedU256,
            new: WrappedU256,
        ) {
            self.env()
                .emit_event(NewCollateralFactor { pool, old, new });
        }

        fn _emit_pool_action_paused_event(&self, pool: AccountId, action: String, paused: bool) {
            self.env().emit_event(PoolActionPaused {
                pool,
                action,
                paused,
            });
        }

        fn _emit_action_paused_event(&self, action: String, paused: bool) {
            self.env().emit_event(ActionPaused { action, paused });
        }

        fn _emit_new_price_oracle_event(&self, old: Option<AccountId>, new: Option<AccountId>) {
            self.env().emit_event(NewPriceOracle { old, new });
        }

        fn _emit_new_flashloan_gateway_event(
            &self,
            old: Option<AccountId>,
            new: Option<AccountId>,
        ) {
            self.env().emit_event(NewFlashloanGateway { old, new });
        }

        fn _emit_new_close_factor_event(&self, old: WrappedU256, new: WrappedU256) {
            self.env().emit_event(NewCloseFactor { old, new });
        }

        fn _emit_new_liquidation_incentive_event(&self, old: WrappedU256, new: WrappedU256) {
            self.env().emit_event(NewLiquidationIncentive { old, new });
        }

        fn _emit_new_borrow_cap_event(&self, pool: AccountId, new: Balance) {
            self.env().emit_event(NewBorrowCap { pool, new });
        }
    }
}
