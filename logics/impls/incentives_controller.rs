// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::traits::incentives_controller::*;

use openbrush::traits::{
    AccountId,
    Balance,
    Storage,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug, Default)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub is_ok: bool,
}

pub trait Internal {
    fn _set_ok(&mut self, is_ok: bool) -> Result<()>;
}

impl<T: Storage<Data>> IncentivesController for T {
    default fn handle_action(
        &mut self,
        _user: AccountId,
        _total_deposit: Balance,
        _total_borrow: Balance,
        _user_deposit: Balance,
        _assetuser_borrow: Balance,
    ) -> Result<()> {
        if self.data::<Data>().is_ok {
            return Ok(())
        }
        return Err(Error::CallerIsNotConfiguredAsset)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _set_ok(&mut self, is_ok: bool) -> Result<()> {
        self.data::<Data>().is_ok = is_ok;
        Ok(())
    }
}
