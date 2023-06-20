#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

/// Definition of Manager Contract
#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use logics::{
        impls::manager::{
            self,
            Internal as ManagerInternal,
        },
        traits::{
            manager::Result,
            types::WrappedU256,
        },
    };
    use openbrush::{
        contracts::access_control::{
            self,
            Internal as AccessControlInternal,
            RoleType,
            DEFAULT_ADMIN_ROLE,
        },
        modifiers,
        traits::Storage,
    };

    const CONTROLLER_ADMIN: RoleType = ink::selector_id!("CONTROLLER_ADMIN");
    const TOKEN_ADMIN: RoleType = ink::selector_id!("TOKEN_ADMIN");
    const BORROW_CAP_GUARDIAN: RoleType = ink::selector_id!("BORROW_CAP_GUARDIAN");
    const PAUSE_GUARDIAN: RoleType = ink::selector_id!("PAUSE_GUARDIAN");

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Storage)]
    pub struct ManagerContract {
        #[storage_field]
        manager: manager::Data,
        #[storage_field]
        access: access_control::Data,
    }

    /// Event: The admin role holder has changed
    #[ink(event)]
    pub struct RoleAdminChanged {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        previous_admin_role: RoleType,
        #[ink(topic)]
        new_admin_role: RoleType,
    }

    /// Event: New role is assigned to the account
    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        grantee: AccountId,
        #[ink(topic)]
        grantor: Option<AccountId>,
    }

    /// Event: The role has been revoked from the account
    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        admin: AccountId,
    }

    /// NOTE: Apply permission control by overriding the Default implementation to use the permission settings in Manager.
    impl manager::Manager for ManagerContract {
        #[ink(message)]
        #[modifiers(access_control::only_role(DEFAULT_ADMIN_ROLE))]
        fn set_controller(&mut self, id: AccountId) -> Result<()> {
            self._set_controller(id)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
        fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
            self._set_price_oracle(new_oracle)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
        fn set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()> {
            self._set_flashloan_gateway(new_flashloan_gateway)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
        fn support_market(&mut self, pool: AccountId, underlying: AccountId) -> Result<()> {
            self._support_market(pool, underlying)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
        fn support_market_with_collateral_factor_mantissa(
            &mut self,
            pool: AccountId,
            underlying: AccountId,
            collateral_factor_mantissa: WrappedU256,
        ) -> Result<()> {
            self._support_market_with_collateral_factor_mantissa(
                pool,
                underlying,
                collateral_factor_mantissa,
            )
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
        fn set_collateral_factor_mantissa(
            &mut self,
            pool: AccountId,
            new_collateral_factor_mantissa: WrappedU256,
        ) -> Result<()> {
            self._set_collateral_factor_mantissa(pool, new_collateral_factor_mantissa)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(PAUSE_GUARDIAN))]
        fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
            self._set_mint_guardian_paused(pool, paused)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(PAUSE_GUARDIAN))]
        fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
            self._set_borrow_guardian_paused(pool, paused)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
        fn set_close_factor_mantissa(
            &mut self,
            new_close_factor_mantissa: WrappedU256,
        ) -> Result<()> {
            self._set_close_factor_mantissa(new_close_factor_mantissa)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
        fn set_liquidation_incentive_mantissa(
            &mut self,
            new_liquidation_incentive_mantissa: WrappedU256,
        ) -> Result<()> {
            self._set_liquidation_incentive_mantissa(new_liquidation_incentive_mantissa)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(BORROW_CAP_GUARDIAN))]
        fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()> {
            self._set_borrow_cap(pool, new_cap)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(TOKEN_ADMIN))]
        fn set_reserve_factor_mantissa(
            &mut self,
            pool: AccountId,
            new_reserve_factor_mantissa: WrappedU256,
        ) -> Result<()> {
            self._set_reserve_factor_mantissa(pool, new_reserve_factor_mantissa)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(TOKEN_ADMIN))]
        fn reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
            self._reduce_reserves(pool, amount)
        }
        #[ink(message)]
        #[modifiers(access_control::only_role(TOKEN_ADMIN))]
        fn sweep_token(&mut self, pool: AccountId, asset: AccountId) -> Result<()> {
            self._sweep_token(pool, asset)
        }
    }

    impl access_control::AccessControl for ManagerContract {}

    impl access_control::Internal for ManagerContract {
        fn _emit_role_admin_changed(
            &mut self,
            role: u32,
            previous_admin_role: u32,
            new_admin_role: u32,
        ) {
            self.env().emit_event(RoleAdminChanged {
                role,
                previous_admin_role,
                new_admin_role,
            })
        }

        fn _emit_role_granted(
            &mut self,
            role: u32,
            grantee: AccountId,
            grantor: Option<AccountId>,
        ) {
            self.env().emit_event(RoleGranted {
                role,
                grantee,
                grantor,
            })
        }

        fn _emit_role_revoked(&mut self, role: u32, account: AccountId, sender: AccountId) {
            self.env().emit_event(RoleRevoked {
                role,
                account,
                admin: sender,
            })
        }
    }

    impl ManagerContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(controller: AccountId) -> Self {
            let mut instance = Self {
                manager: manager::Data { controller },
                access: access_control::Data::default(),
            };
            instance._init_with_caller();
            instance
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
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
            traits::ZERO_ADDRESS,
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
                Ok(Event::RoleGranted(x)) => return x,
                _ => panic!("unexpected event kind: expected RoleGranted event"),
            }
        }
        fn decode_role_revoked_event(event: test::EmittedEvent) -> RoleRevoked {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..]);
            match decoded_event {
                Ok(Event::RoleRevoked(x)) => return x,
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
    }
}
