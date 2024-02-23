use crate::contract::*;
use core::ops::{
    Add,
    Div,
    Mul,
};
use ink::env::{
    test::{
        self,
        recorded_events,
        DefaultAccounts,
        EmittedEvent,
    },
    DefaultEnvironment,
};
use logics::{
    impls::{
        controller::*,
        exp_no_err::exp_scale,
    },
    traits::types::WrappedU256,
};
use openbrush::traits::AccountId;
use primitive_types::U256;
use scale::Decode;

type Event = <ControllerContract as ink::reflect::ContractEventBase>::Type;

fn default_accounts() -> DefaultAccounts<DefaultEnvironment> {
    test::default_accounts::<DefaultEnvironment>()
}
fn set_caller(id: AccountId) {
    test::set_caller::<DefaultEnvironment>(id);
}
fn get_emitted_events() -> Vec<EmittedEvent> {
    recorded_events().collect::<Vec<_>>()
}
fn decode_market_listed_event(event: EmittedEvent) -> MarketListed {
    if let Ok(Event::MarketListed(x)) = <Event as Decode>::decode(&mut &event.data[..]) {
        return x
    }
    panic!("unexpected event kind: expected MarketListed event")
}

#[ink::test]
fn new_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let contract = ControllerContract::new(accounts.bob);
    assert_eq!(contract.markets(), []);
    assert!(!contract.seize_guardian_paused());
    assert!(!contract.transfer_guardian_paused());
    assert_eq!(contract.oracle(), None);
    assert_eq!(contract.manager().unwrap(), accounts.bob);
    assert_eq!(contract.close_factor_mantissa(), WrappedU256::from(0));
    assert_eq!(
        contract.liquidation_incentive_mantissa(),
        WrappedU256::from(0)
    );
}

#[ink::test]
fn mint_allowed_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let pool = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.support_market(pool, underlying).is_ok());
    assert!(contract.mint_allowed(pool, accounts.bob, 0).is_ok());
}

#[ink::test]
fn mint_allowed_fail_when_not_supported() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let contract = ControllerContract::new(accounts.bob);

    let pool = AccountId::from([0x01; 32]);
    assert_eq!(
        contract.mint_allowed(pool, accounts.bob, 0).unwrap_err(),
        Error::MintIsPaused
    );
}

#[ink::test]
fn mint_allowed_fail_when_paused() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let pool = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.support_market(pool, underlying).is_ok());
    assert!(contract.set_mint_guardian_paused(pool, true).is_ok());
    assert_eq!(
        contract.mint_allowed(pool, accounts.bob, 0).unwrap_err(),
        Error::MintIsPaused
    );
}

#[ink::test]
fn borrow_allowed_fail_when_not_supported() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let contract = ControllerContract::new(accounts.bob);

    let pool = AccountId::from([0x01; 32]);
    assert_eq!(
        contract
            .borrow_allowed(pool, accounts.bob, 0, None)
            .unwrap_err(),
        Error::BorrowIsPaused
    );
}

#[ink::test]
fn borrow_allowed_fail_when_paused() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let pool = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.support_market(pool, underlying).is_ok());
    assert!(contract.set_borrow_guardian_paused(pool, true).is_ok());
    assert_eq!(
        contract
            .borrow_allowed(pool, accounts.bob, 0, None)
            .unwrap_err(),
        Error::BorrowIsPaused
    );
}

#[ink::test]
fn liquidate_borrow_allowed_fail() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    // not in market
    let pool1 = AccountId::from([0x01; 32]);
    let pool2 = AccountId::from([0x02; 32]);
    let underlying1 = AccountId::from([0x01; 32]);
    let borrower = AccountId::from([0x04; 32]);
    assert_eq!(
        contract
            .liquidate_borrow_allowed(pool1, pool2, borrower, borrower, 0, None)
            .unwrap_err(),
        Error::MarketNotListed
    );
    assert!(contract.support_market(pool1, underlying1).is_ok());
    assert_eq!(
        contract
            .liquidate_borrow_allowed(pool1, pool2, borrower, borrower, 0, None)
            .unwrap_err(),
        Error::MarketNotListed
    );
}

#[ink::test]
fn seize_allowed_fail() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    // not in market
    let pool1 = AccountId::from([0x01; 32]);
    let pool2 = AccountId::from([0x02; 32]);
    let underlying1 = AccountId::from([0x01; 32]);
    let borrower = AccountId::from([0x04; 32]);
    assert_eq!(
        contract
            .seize_allowed(pool1, pool2, borrower, borrower, 0)
            .unwrap_err(),
        Error::MarketNotListed
    );
    assert!(contract.support_market(pool1, underlying1).is_ok());
    assert_eq!(
        contract
            .seize_allowed(pool1, pool2, borrower, borrower, 0)
            .unwrap_err(),
        Error::MarketNotListed
    );
}

#[ink::test]
fn support_market_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let p1 = AccountId::from([0x01; 32]);
    let underlying1 = AccountId::from([0x01; 32]);
    assert!(contract.support_market(p1, underlying1).is_ok());
    assert_eq!(contract.markets(), [p1]);
    assert_eq!(contract.collateral_factor_mantissa(p1), None);
    assert_eq!(contract.mint_guardian_paused(p1), Some(false));
    assert_eq!(contract.borrow_guardian_paused(p1), Some(false));
    assert_eq!(contract.borrow_cap(p1), Some(0));
    let event = decode_market_listed_event(get_emitted_events()[0].clone());
    assert_eq!(event.pool, p1);

    let p2 = AccountId::from([0x02; 32]);
    let underlying2 = AccountId::from([0x02; 32]);
    assert!(contract.support_market(p2, underlying2).is_ok());
    assert_eq!(contract.markets(), [p1, p2]);
}

#[ink::test]
fn support_market_fails_when_duplicate() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let p1 = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.support_market(p1, underlying).is_ok());
    assert_eq!(
        contract.support_market(p1, underlying).unwrap_err(),
        Error::MarketAlreadyListed
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn support_market_with_collateral_factor_mantissa_fails_by_call_price_oracle_in_set_collateral_factor_mantissa(
) {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let p1 = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);

    let oracle_addr = AccountId::from([0x02; 32]);
    assert_eq!(contract.set_price_oracle(oracle_addr).unwrap(), ());

    contract
        .support_market_with_collateral_factor_mantissa(p1, underlying, WrappedU256::from(1))
        .unwrap();
}

#[ink::test]
fn support_market_with_collateral_factor_mantissa_fails_when_collateral_factor_is_zero() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let p1 = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);
    assert_eq!(
        contract
            .support_market_with_collateral_factor_mantissa(p1, underlying, WrappedU256::from(0))
            .unwrap_err(),
        Error::InvalidCollateralFactor
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_collateral_factor_mantissa_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);
    let pool_addr = AccountId::from([0x01; 32]);
    assert_eq!(contract.collateral_factor_mantissa(pool_addr), None);

    let oracle_addr = AccountId::from([0x02; 32]);
    assert_eq!(contract.set_price_oracle(oracle_addr).unwrap(), ());
    let max = exp_scale().mul(U256::from(90)).div(U256::from(100));
    contract
        .set_collateral_factor_mantissa(pool_addr, WrappedU256::from(max))
        .unwrap_err();
}

#[ink::test]
fn set_collateral_factor_mantissa_fail_when_invalid_value() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);
    let pool_addr = AccountId::from([0x01; 32]);
    assert_eq!(contract.collateral_factor_mantissa(pool_addr), None);

    let max = exp_scale().mul(U256::from(90)).div(U256::from(100));

    assert_eq!(
        contract
            .set_collateral_factor_mantissa(pool_addr, WrappedU256::from(0))
            .unwrap_err(),
        Error::InvalidCollateralFactor
    );
    assert_eq!(
        contract
            .set_collateral_factor_mantissa(pool_addr, WrappedU256::from(max.add(1)))
            .unwrap_err(),
        Error::InvalidCollateralFactor
    );
}

#[ink::test]
fn mint_guardian_paused_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let pool = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);
    assert_eq!(contract.mint_guardian_paused(pool), None);

    assert!(contract.support_market(pool, underlying).is_ok());
    assert_eq!(contract.mint_guardian_paused(pool), Some(false));

    assert!(contract.set_mint_guardian_paused(pool, true).is_ok());
    assert_eq!(contract.mint_guardian_paused(pool), Some(true));
    assert!(contract.set_mint_guardian_paused(pool, false).is_ok());
    assert_eq!(contract.mint_guardian_paused(pool), Some(false));
}

#[ink::test]
fn borrow_guardian_paused_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    let pool = AccountId::from([0x01; 32]);
    let underlying = AccountId::from([0x01; 32]);
    assert_eq!(contract.borrow_guardian_paused(pool), None);

    assert!(contract.support_market(pool, underlying).is_ok());
    assert_eq!(contract.mint_guardian_paused(pool), Some(false));

    assert!(contract.set_borrow_guardian_paused(pool, true).is_ok());
    assert_eq!(contract.borrow_guardian_paused(pool), Some(true));
    assert!(contract.set_borrow_guardian_paused(pool, false).is_ok());
    assert_eq!(contract.borrow_guardian_paused(pool), Some(false));
}

#[ink::test]
fn seize_guardian_paused_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    assert!(!contract.seize_guardian_paused());
    assert!(contract.set_seize_guardian_paused(true).is_ok());
    assert!(contract.seize_guardian_paused());
}

#[ink::test]
fn transfer_guardian_paused_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    assert!(!contract.transfer_guardian_paused());
    assert!(contract.set_transfer_guardian_paused(true).is_ok());
    assert!(contract.transfer_guardian_paused());
}

#[ink::test]
fn assert_manager_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ControllerContract::new(accounts.bob);

    set_caller(accounts.charlie);
    let dummy_id = AccountId::from([0xff; 32]);
    let underlying = AccountId::from([0x01; 32]);
    let admin_funcs: Vec<Result<()>> = vec![
        contract.set_price_oracle(dummy_id),
        contract.support_market(dummy_id, underlying),
        contract.set_manager(dummy_id),
        contract.support_market_with_collateral_factor_mantissa(
            dummy_id,
            underlying,
            WrappedU256::from(0),
        ),
        contract.set_collateral_factor_mantissa(dummy_id, WrappedU256::from(0)),
        contract.set_mint_guardian_paused(dummy_id, true),
        contract.set_borrow_guardian_paused(dummy_id, true),
        contract.set_seize_guardian_paused(true),
        contract.set_transfer_guardian_paused(true),
        contract.set_close_factor_mantissa(WrappedU256::from(0)),
        contract.set_liquidation_incentive_mantissa(WrappedU256::from(0)),
        contract.set_borrow_cap(dummy_id, 0),
    ];
    for func in admin_funcs {
        assert_eq!(func.unwrap_err(), Error::CallerIsNotManager);
    }
}

#[ink::test]
fn set_manager_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    contract.set_manager(accounts.alice).unwrap();
    assert_eq!(contract.pending_manager().unwrap(), accounts.alice);
}

#[ink::test]
fn accept_manager_not_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);

    assert_eq!(
        contract.accept_manager().unwrap_err(),
        Error::PendingManagerIsNotSet
    );
}

#[ink::test]
fn accept_manager_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ControllerContract::new(accounts.bob);
    contract.set_manager(accounts.alice).unwrap();

    set_caller(accounts.alice);
    contract.accept_manager().unwrap();

    assert_eq!(contract.pending_manager(), None);
    assert_eq!(contract.manager().unwrap(), accounts.alice);
}
