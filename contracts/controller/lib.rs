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
            instance.controller.manager = Some(manager);
            instance
        }
    }

    impl Internal for ControllerContract {
        fn _emit_market_listed_event(&self, pool: AccountId) {
            self.env().emit_event(MarketListed { pool });
        }
    }
}
