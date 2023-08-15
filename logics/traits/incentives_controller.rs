// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use openbrush::{
    self,
    traits::{
        AccountId,
        Balance,
    },
};
use scale::{
    Decode,
    Encode,
};

#[openbrush::wrapper]
pub type IncentivesControllerRef = dyn IncentivesController;

#[openbrush::trait_definition]
pub trait IncentivesController {
    /// Called by pools to accrue rewards.
    #[ink(message)]
    fn handle_action(
        &mut self,
        user: AccountId,
        total_deposit: Balance,
        total_borrow: Balance,
        user_deposit: Balance,
        user_borrow: Balance,
    ) -> Result<()>;
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    CallerIsNotConfiguredAsset,
}

pub type Result<T> = core::result::Result<T, Error>;
