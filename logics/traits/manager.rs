use openbrush::traits::AccountId;

#[openbrush::wrapper]
pub type ManagerRef = dyn Manager;

#[openbrush::trait_definition]
pub trait Manager {
    #[ink(message)]
    fn controller(&self) -> AccountId;
    #[ink(message)]
    fn set_controller(&mut self, address: AccountId) -> Result<()>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {}

pub type Result<T> = core::result::Result<T, Error>;
