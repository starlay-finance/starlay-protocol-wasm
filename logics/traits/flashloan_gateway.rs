// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::traits::pool::Error as PoolError;
use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp22::PSP22Error,
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
pub type FlashloanGatewayRef = dyn FlashloanGateway;

#[openbrush::trait_definition]
pub trait FlashloanGateway {
    /// Allows smartcontracts to access the liquidity of the pool within one transaction, as long as the amount taken plus a fee is returned.
    ///  IMPORTANT There are security concerns for developers of flashloan receiver contracts that must be kept into consideration.
    #[ink(message)]
    fn flashloan(
        &self,
        receiver_address: AccountId,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        mods: Vec<u8>,
        on_behalf_of: AccountId,
        params: Vec<u8>,
    ) -> Result<()>;

    /// Returns the fee on flash loans
    #[ink(message)]
    fn flashloan_premium_total(&self) -> u128;

    /// Returns Controller Address
    #[ink(message)]
    fn controller(&self) -> Option<AccountId>;
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    InconsistentFlashloanParams,
    InvalidFlashloanExecutorReturn,
    InvalidFlashloanAmount,
    DuplicatedFlashloanAssets,
    MarketNotListed,
    ControllerIsNotSet,
    PSP22(PSP22Error),
    Pool(PoolError),
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[repr(u8)]
pub enum FlashLoanType {
    None = 0,
    Borrowing = 1,
}

impl From<PSP22Error> for Error {
    fn from(error: PSP22Error) -> Self {
        Error::PSP22(error)
    }
}

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Error::Pool(error)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
