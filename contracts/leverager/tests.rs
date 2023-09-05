use crate::contract::*;
use ink::env::{
    test::{
        self,
        DefaultAccounts,
    },
    DefaultEnvironment,
};
use logics::{
    impls::leverager::Leverager,
    traits::leverager::Error,
};
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
