// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp22::PSP22Ref,
    traits::{
        AccountId,
        Balance,
        Storage,
    },
};

pub use crate::traits::flashloan_receiver::*;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug, Default)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// AccountId of Flashloan Gateway
    pub flashloan_gateway: Option<AccountId>,
    /// For mock only: Set flashloan execution as success or fail
    pub fail_execution: bool,
}

pub trait Internal {
    fn _initialize(&mut self, flashloan_gateway: AccountId);
    fn _set_fail_execution_transfer(&mut self, fail: bool);
    fn _fail_execution_transfer(&self) -> bool;
}

impl<T: Storage<Data>> Internal for T {
    default fn _initialize(&mut self, flashloan_gateway: AccountId) {
        self.data().flashloan_gateway = Some(flashloan_gateway);
    }

    default fn _set_fail_execution_transfer(&mut self, fail: bool) {
        self.data().fail_execution = fail;
    }

    default fn _fail_execution_transfer(&self) -> bool {
        self.data().fail_execution
    }
}

impl<T: Storage<Data>> FlashloanReceiver for T {
    default fn execute_operation(
        &self,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        premiums: Vec<Balance>,
        initiator: AccountId,
        _params: Vec<u8>,
    ) -> bool {
        if self._fail_execution_transfer() {
            return false
        }
        let contract_addr = Self::env().account_id();
        if let Some(gateway) = self.data().flashloan_gateway {
            for index in 0..assets.len() {
                let current_asset = assets[index];
                let transfer_result = PSP22Ref::transfer_from(
                    &current_asset,
                    initiator,
                    contract_addr,
                    premiums[index],
                    Vec::<u8>::new(),
                );
                if transfer_result.is_err() {
                    return false
                }

                let balance = PSP22Ref::balance_of(&current_asset, contract_addr);

                let amount_to_return = amounts[index] + premiums[index];

                if balance < amount_to_return {
                    return false
                }

                let approve_result = PSP22Ref::approve(&current_asset, gateway, amount_to_return);
                if approve_result.is_err() {
                    return false
                }
            }
            return true
        }

        false
    }
}
