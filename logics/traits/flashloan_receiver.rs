// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ink::prelude::vec::Vec;
use openbrush::traits::{
    AccountId,
    Balance,
};

#[openbrush::wrapper]
pub type FlashloanReceiverRef = dyn FlashloanReceiver;

#[openbrush::trait_definition]
pub trait FlashloanReceiver {
    /// Run FlashLoan action
    #[ink(message)]
    fn execute_operation(
        &self,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        premiums: Vec<Balance>,
        initiator: AccountId,
        params: Vec<u8>,
    ) -> bool;
}
