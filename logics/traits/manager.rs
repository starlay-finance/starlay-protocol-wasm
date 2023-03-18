use openbrush::{
    contracts::traits::access_control::AccessControlError,
    traits::{
        AccountId,
        Balance,
    },
};

#[openbrush::wrapper]
pub type ManagerRef = dyn Manager;

#[openbrush::trait_definition]
pub trait Manager {
    #[ink(message)]
    fn controller(&self) -> AccountId;
    #[ink(message)]
    fn set_controller(&mut self, address: AccountId) -> Result<()>;
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
