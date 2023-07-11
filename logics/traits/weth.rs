// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use openbrush::{
    contracts::psp22::extensions::metadata::*,
    traits::Balance,
};

#[openbrush::wrapper]
pub type WETHRef = dyn WETH + PSP22 + PSP22Metadata;

#[openbrush::trait_definition]
pub trait WETH {
    /// Deposit ETH and get WETH instead
    #[ink(message, payable)]
    fn deposit(&mut self) -> Result<(), PSP22Error>;

    /// Withdraw ETH
    #[ink(message)]
    fn withdraw(&mut self, value: Balance) -> Result<(), PSP22Error>;
}
