// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::traits::manager::*;
use crate::traits::{
    controller::{
        ControllerRef,
        Error as ControllerError,
    },
    pool::PoolRef,
    types::WrappedU256,
};
use openbrush::{
    contracts::access_control::{
        self,
        RoleType,
        DEFAULT_ADMIN_ROLE,
    },
    modifiers,
    traits::{
        AccountId,
        Balance,
        Storage,
    },
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// AccountId of Controller
    pub controller: AccountId,
}

pub const CONTROLLER_ADMIN: RoleType = ink::selector_id!("CONTROLLER_ADMIN");
pub const TOKEN_ADMIN: RoleType = ink::selector_id!("TOKEN_ADMIN");
pub const BORROW_CAP_GUARDIAN: RoleType = ink::selector_id!("BORROW_CAP_GUARDIAN");
pub const PAUSE_GUARDIAN: RoleType = ink::selector_id!("PAUSE_GUARDIAN");

pub trait Internal {
    fn _controller(&self) -> AccountId;
    fn _set_controller(&mut self, id: AccountId) -> Result<()>;
    fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;
    fn _set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()>;
    fn _support_market(&mut self, pool: AccountId, underlying: AccountId) -> Result<()>;
    fn _support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        underlying: AccountId,
        collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;
    fn _set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;
    fn _set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;
    fn _set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()>;
    fn _set_close_factor_mantissa(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()>;
    fn _set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()>;
    fn _set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()>;
    fn _set_reserve_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()>;
    fn _reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()>;
    fn _sweep_token(&mut self, pool: AccountId, asset: AccountId) -> Result<()>;
    fn _set_seize_guardian_paused(&mut self, paused: bool) -> Result<()>;
    fn _set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()>;
    fn _set_liquidation_threshold(
        &mut self,
        pool: AccountId,
        liquidation_threshold: u128,
    ) -> Result<()>;
    fn _set_incentives_controller(
        &mut self,
        pool: AccountId,
        incentives_controller: AccountId,
    ) -> Result<()>;
    fn _set_controller_manager(&mut self, manager: AccountId) -> Result<()>;
    fn _accept_controller_manager(&mut self) -> Result<()>;
    fn _set_pool_manager(&mut self, pool: AccountId, manager: AccountId) -> Result<()>;
    fn _accept_pool_manager(&mut self, pool: AccountId) -> Result<()>;
}

impl<T: Storage<Data> + Storage<access_control::Data>> Manager for T {
    // View Function
    default fn controller(&self) -> AccountId {
        self._controller()
    }

    // Default Admin
    #[modifiers(access_control::only_role(DEFAULT_ADMIN_ROLE))]
    default fn set_controller(&mut self, id: AccountId) -> Result<()> {
        self._set_controller(id)
    }

    // For Controller Admin
    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        self._set_price_oracle(new_oracle)
    }

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()> {
        self._set_flashloan_gateway(new_flashloan_gateway)
    }

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn support_market(&mut self, pool: AccountId, underlying: AccountId) -> Result<()> {
        self._support_market(pool, underlying)
    }

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn support_market_with_collateral_factor_mantissa(
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

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_collateral_factor_mantissa(pool, new_collateral_factor_mantissa)
    }

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn set_close_factor_mantissa(
        &mut self,
        new_close_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_close_factor_mantissa(new_close_factor_mantissa)
    }

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_liquidation_incentive_mantissa(new_liquidation_incentive_mantissa)
    }

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn set_controller_manager(&mut self, manager: AccountId) -> Result<()> {
        self._set_controller_manager(manager)
    }

    #[modifiers(access_control::only_role(CONTROLLER_ADMIN))]
    default fn accept_controller_manager(&mut self) -> Result<()> {
        self._accept_controller_manager()
    }

    // For Borrow Cap Admin
    #[modifiers(access_control::only_role(BORROW_CAP_GUARDIAN))]
    default fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()> {
        self._set_borrow_cap(pool, new_cap)
    }

    // For Pause Guardian
    #[modifiers(access_control::only_role(PAUSE_GUARDIAN))]
    default fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._set_mint_guardian_paused(pool, paused)
    }

    #[modifiers(access_control::only_role(PAUSE_GUARDIAN))]
    default fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._set_borrow_guardian_paused(pool, paused)
    }

    #[modifiers(access_control::only_role(PAUSE_GUARDIAN))]
    default fn set_seize_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self._set_seize_guardian_paused(paused)
    }

    #[modifiers(access_control::only_role(PAUSE_GUARDIAN))]
    default fn set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self._set_transfer_guardian_paused(paused)
    }

    // For Pool Admin
    #[modifiers(access_control::only_role(TOKEN_ADMIN))]
    default fn reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
        self._reduce_reserves(pool, amount)
    }

    #[modifiers(access_control::only_role(TOKEN_ADMIN))]
    default fn sweep_token(&mut self, pool: AccountId, asset: AccountId) -> Result<()> {
        self._sweep_token(pool, asset)
    }

    #[modifiers(access_control::only_role(TOKEN_ADMIN))]
    default fn set_liquidation_threshold(
        &mut self,
        pool: AccountId,
        liquidation_threshold: u128,
    ) -> Result<()> {
        self._set_liquidation_threshold(pool, liquidation_threshold)
    }

    #[modifiers(access_control::only_role(TOKEN_ADMIN))]
    default fn set_reserve_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_reserve_factor_mantissa(pool, new_reserve_factor_mantissa)
    }

    #[modifiers(access_control::only_role(TOKEN_ADMIN))]
    default fn set_incentives_controller(
        &mut self,
        pool: AccountId,
        incentives_controller: AccountId,
    ) -> Result<()> {
        self._set_incentives_controller(pool, incentives_controller)
    }

    #[modifiers(access_control::only_role(TOKEN_ADMIN))]
    default fn set_pool_manager(&mut self, pool: AccountId, manager: AccountId) -> Result<()> {
        self._set_pool_manager(pool, manager)
    }

    #[modifiers(access_control::only_role(TOKEN_ADMIN))]
    default fn accept_pool_manager(&mut self, pool: AccountId) -> Result<()> {
        self._accept_pool_manager(pool)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _controller(&self) -> AccountId {
        self.data().controller
    }
    default fn _set_controller(&mut self, id: AccountId) -> Result<()> {
        self.data().controller = id;
        Ok(())
    }
    default fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        ControllerRef::set_price_oracle(&self._controller(), new_oracle)?;
        Ok(())
    }
    default fn _set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()> {
        ControllerRef::set_flashloan_gateway(&self._controller(), new_flashloan_gateway)?;
        Ok(())
    }
    default fn _support_market(&mut self, pool: AccountId, underlying: AccountId) -> Result<()> {
        ControllerRef::support_market(&self._controller(), pool, underlying)?;
        Ok(())
    }
    default fn _set_controller_manager(&mut self, manager: AccountId) -> Result<()> {
        ControllerRef::set_manager(&self._controller(), manager)?;
        Ok(())
    }

    default fn _accept_controller_manager(&mut self) -> Result<()> {
        ControllerRef::accept_manager(&self._controller())?;
        Ok(())
    }

    default fn _support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        underlying: AccountId,
        collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        ControllerRef::support_market_with_collateral_factor_mantissa(
            &self._controller(),
            pool,
            underlying,
            collateral_factor_mantissa,
        )?;
        Ok(())
    }
    default fn _set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        ControllerRef::set_collateral_factor_mantissa(
            &self._controller(),
            pool,
            new_collateral_factor_mantissa,
        )?;
        Ok(())
    }
    default fn _set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        ControllerRef::set_mint_guardian_paused(&self._controller(), pool, paused)?;
        Ok(())
    }
    default fn _set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        ControllerRef::set_borrow_guardian_paused(&self._controller(), pool, paused)?;
        Ok(())
    }
    default fn _set_close_factor_mantissa(
        &mut self,
        new_close_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        ControllerRef::set_close_factor_mantissa(&self._controller(), new_close_factor_mantissa)?;
        Ok(())
    }
    default fn _set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        ControllerRef::set_liquidation_incentive_mantissa(
            &self._controller(),
            new_liquidation_incentive_mantissa,
        )?;
        Ok(())
    }
    default fn _set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()> {
        ControllerRef::set_borrow_cap(&self._controller(), pool, new_cap)?;
        Ok(())
    }
    default fn _set_reserve_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        PoolRef::set_reserve_factor_mantissa(&pool, new_reserve_factor_mantissa)?;
        Ok(())
    }
    default fn _reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
        PoolRef::reduce_reserves(&pool, amount)?;
        Ok(())
    }
    default fn _sweep_token(&mut self, pool: AccountId, asset: AccountId) -> Result<()> {
        PoolRef::sweep_token(&pool, asset)?;
        Ok(())
    }
    default fn _set_seize_guardian_paused(&mut self, paused: bool) -> Result<()> {
        ControllerRef::set_seize_guardian_paused(&self._controller(), paused)?;
        Ok(())
    }
    default fn _set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()> {
        ControllerRef::set_transfer_guardian_paused(&self._controller(), paused)?;
        Ok(())
    }
    default fn _set_liquidation_threshold(
        &mut self,
        pool: AccountId,
        liquidation_threshold: u128,
    ) -> Result<()> {
        PoolRef::set_liquidation_threshold(&pool, liquidation_threshold)?;
        Ok(())
    }
    default fn _set_incentives_controller(
        &mut self,
        pool: AccountId,
        incentives_controller: AccountId,
    ) -> Result<()> {
        PoolRef::set_incentives_controller(&pool, incentives_controller)?;
        Ok(())
    }
    default fn _set_pool_manager(&mut self, pool: AccountId, manager: AccountId) -> Result<()> {
        let controller = self.data().controller;
        let is_listed: bool = ControllerRef::is_listed(&controller, pool);
        if !is_listed {
            return Err(Error::from(ControllerError::MarketNotListed))
        }

        PoolRef::set_manager(&pool, manager)?;
        Ok(())
    }
    default fn _accept_pool_manager(&mut self, pool: AccountId) -> Result<()> {
        let controller = self.data().controller;
        let is_listed: bool = ControllerRef::is_listed(&controller, pool);
        if !is_listed {
            return Err(Error::from(ControllerError::MarketNotListed))
        }

        PoolRef::accept_manager(&pool)?;
        Ok(())
    }
}
