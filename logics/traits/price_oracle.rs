use openbrush::traits::AccountId;

#[openbrush::wrapper]
pub type PriceOracleRef = dyn PriceOracle;

#[openbrush::trait_definition]
pub trait PriceOracle {
    #[ink(message)]
    fn get_price(&self, asset: AccountId) -> Option<u128>;
    #[ink(message)]
    fn get_underlying_price(&self, asset: AccountId) -> Option<u128>;
    #[ink(message)]
    fn set_fixed_price(&mut self, asset: AccountId, value: u128) -> Result<()>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {}

pub type Result<T> = core::result::Result<T, Error>;
