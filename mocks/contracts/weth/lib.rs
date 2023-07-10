// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use logics::impls::weth::{
        Internal,
        *,
    };
    use openbrush::{
        contracts::psp22::extensions::{
            metadata::*,
            mintable::*,
        },
        traits::Storage,
    };

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        dst: AccountId,
        wad: Balance,
    }

    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        src: AccountId,
        wad: Balance,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct WETHContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl PSP22 for WETHContract {}
    impl PSP22Metadata for WETHContract {}
    impl PSP22Mintable for WETHContract {}
    impl WETH for WETHContract {}

    impl Internal for WETHContract {
        fn _emit_deposit_event(&mut self, dst: AccountId, wad: Balance) {
            self.env().emit_event(Deposit { dst, wad });
        }
        fn _emit_withdraw_event(&mut self, src: AccountId, wad: Balance) {
            self.env().emit_event(Withdraw { src, wad });
        }
    }

    impl WETHContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut _instance = Self::default();
            _instance.metadata.name = Some("Wrapped Astar".into());
            _instance.metadata.symbol = Some("WASTR".into());
            _instance.metadata.decimals = 18;
            _instance
        }
    }
}
