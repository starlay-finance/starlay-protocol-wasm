#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of Manager Contract
#[openbrush::contract]
pub mod manager {
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

    pub const CONTROLLER_ADMIN: RoleType = ink::selector_id!("CONTROLLER_ADMIN");
    pub const TOKEN_ADMIN: RoleType = ink::selector_id!("TOKEN_ADMIN");
    pub const BORROW_CAP_GUARDIAN: RoleType = ink::selector_id!("BORROW_CAP_GUARDIAN");
    pub const PAUSE_GUARDIAN: RoleType = ink::selector_id!("PAUSE_GUARDIAN");

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
}
