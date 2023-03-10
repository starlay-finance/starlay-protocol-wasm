use openbrush::traits::Balance;

#[openbrush::wrapper]
pub type InterestRateModelRef = dyn InterestRateModel;

#[openbrush::trait_definition]
pub trait InterestRateModel {
    #[ink(message)]
    fn get_borrow_rate(&self, cash: Balance, borrows: Balance, reserves: Balance) -> u128;

    #[ink(message)]
    fn get_supply_rate(
        &self,
        cash: Balance,
        borrows: Balance,
        reserve_factor_mantissa: Balance,
    ) -> u128;
}
