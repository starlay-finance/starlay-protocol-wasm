use openbrush::traits::AccountId;

#[openbrush::wrapper]
pub type PriceOracleRef = dyn PriceOracle;

#[openbrush::trait_definition]
pub trait PriceOracle {
    #[ink(message)]
    fn get_price(&self, asset: AccountId) -> u128;
    #[ink(message)]
    fn get_underlying_price(&self, asset: AccountId) -> u128;
}
