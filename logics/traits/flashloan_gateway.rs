use crate::traits::pool::Error as PoolError;
use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp22::PSP22Error,
    traits::{
        AccountId,
        Balance,
    },
};

#[openbrush::wrapper]
pub type FlashloanGatewayRef = dyn FlashloanGateway;

#[openbrush::trait_definition]
pub trait FlashloanGateway {
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

    #[ink(message)]
    fn flashloan_premium_total(&self) -> u128;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    InconsistentFlashloanParams,
    InvalidFlashloanExecutorReturn,
    MarketNotListed,
    PSP22(PSP22Error),
    Pool(PoolError),
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
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
