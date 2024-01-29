// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of Manager Contract
#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use logics::impls::manager;
    use openbrush::{
        contracts::access_control::{
            self,
            Internal as AccessControlInternal,
            RoleType,
        },
        traits::Storage,
    };

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
        pub role: RoleType,
        #[ink(topic)]
        pub previous_admin_role: RoleType,
        #[ink(topic)]
        pub new_admin_role: RoleType,
    }

    /// Event: New role is assigned to the account
    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        pub role: RoleType,
        #[ink(topic)]
        pub grantee: AccountId,
        #[ink(topic)]
        pub grantor: Option<AccountId>,
    }

    /// Event: The role has been revoked from the account
    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        pub role: RoleType,
        #[ink(topic)]
        pub account: AccountId,
        #[ink(topic)]
        pub admin: AccountId,
    }

    impl manager::Manager for ManagerContract {}

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
}
