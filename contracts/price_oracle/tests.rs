use crate::price_oracle::*;
use ink::env::{
    test::{
        self,
        DefaultAccounts,
    },
    DefaultEnvironment,
};
use logics::impls::price_oracle::*;
use openbrush::traits::AccountId;

fn default_accounts() -> DefaultAccounts<DefaultEnvironment> {
    test::default_accounts::<DefaultEnvironment>()
}
fn set_caller(id: AccountId) {
    test::set_caller::<DefaultEnvironment>(id);
}

#[ink::test]
fn new_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let _contract = PriceOracleContract::new();
}

#[ink::test]
fn set_fixed_price_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = PriceOracleContract::new();

    let asset_addr = AccountId::from([0x01; 32]);
    assert!(contract
        .set_fixed_price(asset_addr, PRICE_PRECISION * 101 / 100)
        .is_ok());
    assert_eq!(
        contract.get_price(asset_addr),
        Some(PRICE_PRECISION * 101 / 100)
    )
}
