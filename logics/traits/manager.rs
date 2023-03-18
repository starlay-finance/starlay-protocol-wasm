use openbrush::{
    contracts::traits::access_control::AccessControlError,
    traits::{
        AccountId,
        Balance,
    },
};

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type ManagerRef = dyn Manager;

#[openbrush::trait_definition]
pub trait Manager {
    #[ink(message)]
    fn controller(&self) -> AccountId;
    #[ink(message)]
    fn set_controller(&mut self, address: AccountId) -> Result<()>;
    #[ink(message)]
    fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;
    #[ink(message)]
    fn support_market(&mut self, pool: AccountId) -> Result<()>;
    #[ink(message)]
    fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;
    #[ink(message)]
    fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;
    #[ink(message)]
    fn set_close_factor_mantissa(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()>;
    #[ink(message)]
    fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()>;
    #[ink(message)]
    fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()>;
    #[ink(message)]
    fn reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    AccessControl(AccessControlError),
}

impl From<AccessControlError> for Error {
    fn from(error: AccessControlError) -> Self {
        Error::AccessControl(error)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
