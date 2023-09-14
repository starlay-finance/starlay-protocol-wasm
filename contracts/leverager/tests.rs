use crate::contract::*;
use ink::env::{
    test::{
        self,
        DefaultAccounts,
    },
    DefaultEnvironment,
};
use logics::impls::leverager::*;
use openbrush::traits::AccountId;
use primitive_types::U256;

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

    let contract = LeveragerContract::new(accounts.bob);
    assert_eq!(contract.manager(), Some(accounts.bob));
}

#[ink::test]
fn initialize_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = LeveragerContract::new(accounts.bob);

    let controller = AccountId::from([0x01; 32]);
    let price_oracle = AccountId::from([0x02; 32]);
    let weth = AccountId::from([0x03; 32]);

    let result = contract.initialize(Some(controller), Some(price_oracle), Some(weth));

    assert!(result.is_ok());

    assert_eq!(contract.controller(), Some(controller));
    assert_eq!(contract.price_oracle(), Some(price_oracle));
    assert_eq!(contract.weth_address(), Some(weth));
}

#[ink::test]
fn initialize_works_only_manager() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = LeveragerContract::new(accounts.bob);

    let controller = AccountId::from([0x01; 32]);
    let price_oracle = AccountId::from([0x02; 32]);
    let weth = AccountId::from([0x03; 32]);

    set_caller(accounts.alice);
    let result = contract.initialize(Some(controller), Some(price_oracle), Some(weth));

    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), Error::CallerIsNotManager);
}

#[ink::test]
fn all_methods_not_working_without_initialize() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let contract = LeveragerContract::new(accounts.bob);

    let available_borrow = contract.get_available_borrows(accounts.bob);
    assert!(available_borrow.is_none());

    let asset = AccountId::from([0x01; 32]);
    let health_factor: U256 = contract.get_health_factor(accounts.bob, asset, 0);
    assert_eq!(health_factor, U256::from(0));

    let withdrawable = contract.withdrawable(accounts.bob, asset);
    assert!(withdrawable.is_none());

    let withdrawable_amount: U256 = contract.withdrawable_amount(accounts.bob, asset);
    assert_eq!(withdrawable_amount, U256::from(0));

    let loan_to_value = contract.loan_to_value(asset);
    assert_eq!(loan_to_value, 0);

    let liquidation_threshold = contract.liquidation_threshold(asset);
    assert_eq!(liquidation_threshold, 0);
}
