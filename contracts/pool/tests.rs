use crate::contract::*;
use ink::{
    env::{
        test::{
            self,
            DefaultAccounts,
        },
        DefaultEnvironment,
    },
    prelude::vec::Vec,
};
use logics::{
    impls::{
        exp_no_err::exp_scale,
        pool::*,
    },
    traits::types::WrappedU256,
};
use openbrush::{
    contracts::psp22::PSP22,
    traits::AccountId,
};
use primitive_types::U256;
use std::ops::{
    Add,
    Div,
};

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

    let underlying = AccountId::from([0x01; 32]);
    let controller = AccountId::from([0x02; 32]);
    let rate_model = AccountId::from([0x03; 32]);
    let incentives_controller = AccountId::from([0x04; 32]);
    let initial_exchange_rate_mantissa = WrappedU256::from(exp_scale());
    let liquidation_threshold = 10000;
    let contract = PoolContract::new(
        Some(incentives_controller),
        underlying,
        controller,
        rate_model,
        accounts.bob,
        initial_exchange_rate_mantissa,
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );
    assert_eq!(contract.underlying(), Some(underlying));
    assert_eq!(contract.controller(), Some(controller));
    assert_eq!(contract.manager(), Some(accounts.bob));
    assert_eq!(
        contract.incentives_controller(),
        Some(incentives_controller)
    );
    assert_eq!(
        contract.initial_exchange_rate_mantissa(),
        initial_exchange_rate_mantissa
    );
    assert_eq!(
        contract.reserve_factor_mantissa(),
        WrappedU256::from(U256::from(0))
    );
    assert_eq!(contract.total_borrows(), 0);
    assert_eq!(contract.liquidation_threshold(), liquidation_threshold);
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn transfer_works_overridden() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    contract.transfer(accounts.charlie, 0, Vec::new()).unwrap();
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn transfer_from_works_overridden() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    contract
        .transfer_from(accounts.bob, accounts.charlie, 0, Vec::new())
        .unwrap();
}

#[ink::test]
fn set_controller_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let dummy_id = AccountId::from([0x01; 32]);
    let dummy_id1 = AccountId::from([0x02; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    contract.set_controller(dummy_id1).unwrap();
    assert_eq!(contract.controller(), Some(dummy_id1));
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn add_reserves_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    contract.add_reserves(0).unwrap()
}

#[ink::test]
fn set_reserve_factor_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );
    assert_eq!(contract.reserve_factor_mantissa(), WrappedU256::from(0));
    let half_exp_scale = exp_scale().div(2);
    assert!(contract
        .set_reserve_factor_mantissa(WrappedU256::from(half_exp_scale))
        .is_ok());
    assert_eq!(
        contract.reserve_factor_mantissa(),
        WrappedU256::from(half_exp_scale)
    );
    let over_exp_scale = exp_scale().add(1);
    assert_eq!(
        contract
            .set_reserve_factor_mantissa(WrappedU256::from(over_exp_scale))
            .unwrap_err(),
        Error::SetReserveFactorBoundsCheck
    );
}

#[ink::test]
fn assert_manager_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    set_caller(accounts.charlie);
    let admin_funcs: Vec<Result<()>> = vec![
        contract.reduce_reserves(100),
        contract.sweep_token(dummy_id),
        contract.set_reserve_factor_mantissa(WrappedU256::from(0)),
    ];
    for func in admin_funcs {
        assert_eq!(func.unwrap_err(), Error::CallerIsNotManager);
    }
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_liquidation_threshold_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let dummy_id = AccountId::from([0x01; 32]);
    let mut liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    liquidation_threshold = 8000;
    let _ = contract.set_liquidation_threshold(liquidation_threshold);
    assert_eq!(contract.liquidation_threshold(), liquidation_threshold);
}

#[ink::test]
fn set_manager_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    contract.set_manager(accounts.alice).unwrap();
    assert_eq!(contract.pending_manager().unwrap(), accounts.alice);
}

#[ink::test]
fn accept_manager_not_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );

    assert_eq!(
        contract.accept_manager().unwrap_err(),
        Error::PendingManagerIsNotSet
    );
}

#[ink::test]
fn accept_manager_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let dummy_id = AccountId::from([0x01; 32]);
    let liquidation_threshold = 10000;
    let mut contract = PoolContract::new(
        Some(dummy_id),
        dummy_id,
        dummy_id,
        dummy_id,
        accounts.bob,
        WrappedU256::from(U256::from(0)),
        liquidation_threshold,
        String::from("Token Name"),
        String::from("symbol"),
        8,
    );
    contract.set_manager(accounts.alice).unwrap();

    set_caller(accounts.alice);
    contract.accept_manager().unwrap();

    assert_eq!(contract.pending_manager(), None);
    assert_eq!(contract.manager().unwrap(), accounts.alice);
}
