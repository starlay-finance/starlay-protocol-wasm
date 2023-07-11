// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::traits::weth::*;
use openbrush::{
    contracts::psp22::extensions::metadata::*,
    traits::{
        AccountId,
        Balance,
        Storage,
    },
};

impl<T> WETH for T
where
    T: Storage<psp22::Data> + Storage<metadata::Data>,
    T: Internal,
{
    default fn deposit(&mut self) -> Result<(), PSP22Error> {
        let caller = Self::env().caller();
        let transferred_value = Self::env().transferred_value();
        let mint_result = self._mint_to(caller, transferred_value);
        if mint_result.is_err() {
            return mint_result
        }
        self._emit_deposit_event(caller, transferred_value);
        Ok(())
    }

    default fn withdraw(&mut self, value: Balance) -> Result<(), PSP22Error> {
        let caller = Self::env().caller();
        if self.balance_of(caller) < value {
            return Err(PSP22Error::InsufficientBalance)
        }
        let burn_result = self._burn_from(caller, value);
        if burn_result.is_err() {
            return burn_result
        }
        let transfer_result = Self::env().transfer(caller, value);
        if transfer_result.is_err() {
            return Err(PSP22Error::Custom("Cannot send ASTR.".into()))
        }
        self._emit_withdraw_event(caller, value);
        Ok(())
    }
}

pub trait Internal {
    fn _emit_deposit_event(&mut self, dst: AccountId, wad: Balance);
    fn _emit_withdraw_event(&mut self, src: AccountId, wad: Balance);
}
