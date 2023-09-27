// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use openbrush::traits::AccountId;
use scale::{
    Decode,
    Encode,
};

#[openbrush::wrapper]
pub type PriceOracleRef = dyn PriceOracle;

/// Trait defines the functions that a PriceOracle must implement.
/// A PriceOracle is responsible for providing the current market price of an asset.
#[openbrush::trait_definition]
pub trait PriceOracle {
    /// Returns the current price for the given asset, if available.
    #[ink(message)]
    fn get_price(&self, asset: AccountId) -> Option<u128>;

    /// Returns the underlying price of the given pool, if available.
    #[ink(message)]
    fn get_underlying_price(&self, pool: AccountId) -> Option<u128>;

    /// Sets a fixed price for the given asset.
    #[ink(message)]
    fn set_fixed_price(&mut self, asset: AccountId, value: u128) -> Result<()>;
}

/// Custom error definitions for PriceOracle
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {}

pub type Result<T> = core::result::Result<T, Error>;
