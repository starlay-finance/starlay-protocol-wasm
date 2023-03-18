#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

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
        traits::manager::Result,
    };
    use openbrush::{
        contracts::access_control::{
            self,
            Internal as AccessControlInternal,
            RoleType,
        },
        modifiers,
        traits::{
            Storage,
            ZERO_ADDRESS,
        },
    };

    const CONTROLLER_ADMIN: RoleType = ink::selector_id!("CONTROLLER_ADMIN");
    const TOKEN_ADMIN: RoleType = ink::selector_id!("TOKEN_ADMIN");
    const BORROW_CAP_GUARDIAN: RoleType = ink::selector_id!("BORROW_CAP_GUARDIAN");
    const PAUSE_GUARDIAN: RoleType = ink::selector_id!("PAUSE_GUARDIAN");

    #[ink(storage)]
    #[derive(Storage)]
    pub struct ManagerContract {
        #[storage_field]
        manager: manager::Data,
        #[storage_field]
        access: access_control::Data,
    }

    #[ink(event)]
    pub struct RoleAdminChanged {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        previous_admin_role: RoleType,
        #[ink(topic)]
        new_admin_role: RoleType,
    }

    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        grantee: AccountId,
        #[ink(topic)]
        grantor: Option<AccountId>,
    }

    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        admin: AccountId,
    }

    impl manager::Manager for ManagerContract {
        #[ink(message)]
        #[modifiers(access_control::only_role(TOKEN_ADMIN))]
        fn reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
            self._reduce_reserves(pool, amount)
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
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self {
                manager: manager::Data {
                    controller: ZERO_ADDRESS.into(),
                },
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
            traits::manager::Error,
        };
        use openbrush::contracts::access_control::{
            AccessControl,
            AccessControlError,
            DEFAULT_ADMIN_ROLE,
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

        #[ink::test]
        fn new_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let contract = ManagerContract::new();

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
        #[should_panic(
            expected = "not implemented: off-chain environment does not support contract invocation"
        )]
        fn reduce_reserves_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ManagerContract::new();
            assert!(contract.grant_role(TOKEN_ADMIN, accounts.bob).is_ok());
            contract.reduce_reserves(ZERO_ADDRESS.into(), 100).unwrap();
        }

        #[ink::test]
        fn reduce_reserves_fails_by_no_authority() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let mut contract = ManagerContract::new();
            assert_eq!(
                contract
                    .reduce_reserves(ZERO_ADDRESS.into(), 100)
                    .unwrap_err(),
                Error::AccessControl(AccessControlError::MissingRole)
            );

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
    }
}
