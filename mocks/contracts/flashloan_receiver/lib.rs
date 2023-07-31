// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
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

        #[ink(message)]
        pub fn set_fail_execution_transfer(&mut self, fail: bool) {
            self._set_fail_execution_transfer(fail);
        }

        #[ink(message)]
        pub fn fail_execution_transfer(&self) -> bool {
            self._fail_execution_transfer()
        }
    }
}
