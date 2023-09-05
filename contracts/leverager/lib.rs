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

/// Definition of Leverager Contract
#[openbrush::contract]
pub mod contract {
    use logics::{
        impls::leverager::*,
        traits::leverager::Result,
    };
    use openbrush::traits::Storage;

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Storage, Default)]
    pub struct LeveragerContract {
        #[storage_field]
        leverager: Data,
    }

    impl Leverager for LeveragerContract {}
    impl Internal for LeveragerContract {}

    impl LeveragerContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(manager: AccountId) -> Self {
            let mut instance: LeveragerContract = Default::default();
            instance.leverager.manager = Some(manager);
            instance
        }

        #[ink(message, payable, selector = _)]
        pub fn fund(&self) -> Result<()> {
            Ok(())
        }
    }
}
