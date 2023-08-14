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
    use logics::impls::incentives_controller::*;

    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct IncentivesControllerContract {
        #[storage_field]
        data: Data,
    }

    impl IncentivesController for IncentivesControllerContract {
        #[ink(message)]
        fn handle_action(
            &mut self,
            _user: AccountId,
            _total_deposit: Balance,
            _total_borrow: Balance,
            _user_deposit: Balance,
            _user_borrow: Balance,
        ) -> Result<()> {
            Ok(())
        }
    }

    impl IncentivesControllerContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[ink(message)]
        pub fn set_ok(&mut self, is_ok: bool) -> Result<()> {
            self._set_ok(is_ok)
        }
    }
}
