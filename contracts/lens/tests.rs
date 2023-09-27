use crate::contract::*;
use ink::env::{
    test::{
        self,
        DefaultAccounts,
    },
    DefaultEnvironment,
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

    let _contract = LensContract::new();
}
