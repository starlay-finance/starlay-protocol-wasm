use crate::manager::*;
use ink::env::{
    test::{
        self,
        DefaultAccounts,
    },
    DefaultEnvironment,
};
use logics::{
    impls::manager::Manager,
    traits::{
        manager::Error,
        types::WrappedU256,
    },
};
use openbrush::{
    contracts::access_control::{
        AccessControl,
        AccessControlError,
        DEFAULT_ADMIN_ROLE,
    },
    traits::{
        AccountId,
        ZERO_ADDRESS,
    },
};

type Event = <ManagerContract as ink::reflect::ContractEventBase>::Type;

fn default_accounts() -> DefaultAccounts<DefaultEnvironment> {
    test::default_accounts::<DefaultEnvironment>()
}
fn set_caller(id: AccountId) {
    test::set_caller::<DefaultEnvironment>(id);
}
fn get_emitted_events() -> Vec<test::EmittedEvent> {
    test::recorded_events().collect::<Vec<_>>()
}
fn decode_role_granted_event(event: test::EmittedEvent) -> RoleGranted {
    let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..]);
    match decoded_event {
        Ok(Event::RoleGranted(x)) => x,
        _ => panic!("unexpected event kind: expected RoleGranted event"),
    }
}
fn decode_role_revoked_event(event: test::EmittedEvent) -> RoleRevoked {
    let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..]);
    match decoded_event {
        Ok(Event::RoleRevoked(x)) => x,
        _ => panic!("unexpected event kind: expected RoleRevoked event"),
    }
}

#[ink::test]
fn new_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let contract = ManagerContract::new(ZERO_ADDRESS.into());

    assert_eq!(contract.controller(), ZERO_ADDRESS.into());
    assert!(contract.has_role(DEFAULT_ADMIN_ROLE, accounts.bob));
    let events = get_emitted_events();
    assert_eq!(events.len(), 1);
    let event = decode_role_granted_event(events[0].clone());
    assert_eq!(event.role, DEFAULT_ADMIN_ROLE);
    assert_eq!(event.grantee, accounts.bob);
    assert_eq!(event.grantor, None);
}

#[ink::test]
fn events_in_access_control_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.has_role(DEFAULT_ADMIN_ROLE, accounts.bob));

    {
        assert!(contract
            .grant_role(CONTROLLER_ADMIN, accounts.alice)
            .is_ok());
        let events = get_emitted_events();
        assert_eq!(events.len(), 2);
        let event = decode_role_granted_event(events[1].clone());
        assert_eq!(event.role, CONTROLLER_ADMIN);
        assert_eq!(event.grantee, accounts.alice);
        assert_eq!(event.grantor, Some(accounts.bob));
    }
    {
        assert!(contract
            .revoke_role(CONTROLLER_ADMIN, accounts.alice)
            .is_ok());
        let events = get_emitted_events();
        assert_eq!(events.len(), 3);
        let event = decode_role_revoked_event(events[2].clone());
        assert_eq!(event.role, CONTROLLER_ADMIN);
        assert_eq!(event.account, accounts.alice);
        assert_eq!(event.admin, accounts.bob);
    }
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_price_oracle_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    contract.set_price_oracle(ZERO_ADDRESS.into()).unwrap();
}
#[ink::test]
fn set_price_oracle_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert_eq!(
        contract.set_price_oracle(ZERO_ADDRESS.into()).unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn support_market_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    contract
        .support_market(ZERO_ADDRESS.into(), underlying)
        .unwrap();
}
#[ink::test]
fn support_market_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert_eq!(
        contract
            .support_market(ZERO_ADDRESS.into(), underlying)
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn support_market_with_collateral_factor_mantissa_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    contract
        .support_market_with_collateral_factor_mantissa(
            ZERO_ADDRESS.into(),
            underlying,
            WrappedU256::from(0),
        )
        .unwrap();
}
#[ink::test]
fn support_market_with_collateral_factor_mantissa_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    let underlying = AccountId::from([0x01; 32]);
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert_eq!(
        contract
            .support_market_with_collateral_factor_mantissa(
                ZERO_ADDRESS.into(),
                underlying,
                WrappedU256::from(0)
            )
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_collateral_factor_mantissa_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    contract
        .set_collateral_factor_mantissa(ZERO_ADDRESS.into(), WrappedU256::from(0))
        .unwrap();
}
#[ink::test]
fn set_collateral_factor_mantissa_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert_eq!(
        contract
            .set_collateral_factor_mantissa(ZERO_ADDRESS.into(), WrappedU256::from(0))
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_mint_guardian_paused_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    contract
        .set_mint_guardian_paused(ZERO_ADDRESS.into(), true)
        .unwrap();
}
#[ink::test]
fn set_mint_guardian_paused_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert_eq!(
        contract
            .set_mint_guardian_paused(ZERO_ADDRESS.into(), true)
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_borrow_guardian_paused_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    contract
        .set_borrow_guardian_paused(ZERO_ADDRESS.into(), true)
        .unwrap();
}
#[ink::test]
fn set_borrow_guardian_paused_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert_eq!(
        contract
            .set_borrow_guardian_paused(ZERO_ADDRESS.into(), true)
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_close_factor_mantissa_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    contract
        .set_close_factor_mantissa(WrappedU256::from(0))
        .unwrap();
}
#[ink::test]
fn set_close_factor_mantissa_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert_eq!(
        contract
            .set_close_factor_mantissa(WrappedU256::from(0))
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_liquidation_incentive_mantissa_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    contract
        .set_liquidation_incentive_mantissa(WrappedU256::from(0))
        .unwrap();
}
#[ink::test]
fn set_liquidation_incentive_mantissa_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert_eq!(
        contract
            .set_liquidation_incentive_mantissa(WrappedU256::from(0))
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_borrow_cap_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    contract.set_borrow_cap(ZERO_ADDRESS.into(), 0).unwrap();
}
#[ink::test]
fn set_borrow_cap_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert_eq!(
        contract.set_borrow_cap(ZERO_ADDRESS.into(), 0).unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn set_reserve_factor_mantissa_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    contract
        .set_reserve_factor_mantissa(ZERO_ADDRESS.into(), WrappedU256::from(0))
        .unwrap();
}
#[ink::test]
fn set_reserve_factor_mantissa_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert_eq!(
        contract
            .set_reserve_factor_mantissa(ZERO_ADDRESS.into(), WrappedU256::from(0))
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn reduce_reserves_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    contract.reduce_reserves(ZERO_ADDRESS.into(), 100).unwrap();
}
#[ink::test]
fn reduce_reserves_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert_eq!(
        contract
            .reduce_reserves(ZERO_ADDRESS.into(), 100)
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}

#[ink::test]
#[should_panic(
    expected = "not implemented: off-chain environment does not support contract invocation"
)]
fn sweep_token_works() {
    let accounts = default_accounts();
    set_caller(accounts.bob);
    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
    contract
        .sweep_token(ZERO_ADDRESS.into(), ZERO_ADDRESS.into())
        .unwrap();
}
#[ink::test]
fn sweep_token_fails_by_no_authority() {
    let accounts = default_accounts();
    set_caller(accounts.bob);

    let mut contract = ManagerContract::new(ZERO_ADDRESS.into());
    assert!(contract.grant_role(CONTROLLER_ADMIN, accounts.bob).is_ok());
    assert!(contract
        .grant_role(BORROW_CAP_GUARDIAN, accounts.bob)
        .is_ok());
    assert!(contract.grant_role(PAUSE_GUARDIAN, accounts.bob).is_ok());
    assert_eq!(
        contract
            .sweep_token(ZERO_ADDRESS.into(), ZERO_ADDRESS.into())
            .unwrap_err(),
        Error::AccessControl(AccessControlError::MissingRole)
    );
}
