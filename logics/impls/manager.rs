pub use crate::traits::manager::*;
use crate::traits::{
    controller::ControllerRef,
    pool::PoolRef,
    types::WrappedU256,
};
use openbrush::traits::{
    AccountId,
    Balance,
    Storage,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub controller: AccountId,
}

pub trait Internal {
    fn _controller(&self) -> AccountId;
    fn _set_controller(&mut self, id: AccountId) -> Result<()>;
    fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;
    fn _support_market(&mut self, pool: AccountId) -> Result<()>;
    fn _support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
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
}

impl<T: Storage<Data>> Manager for T {
    default fn controller(&self) -> AccountId {
        self._controller()
    }
    default fn set_controller(&mut self, id: AccountId) -> Result<()> {
        self._set_controller(id)
    }
    default fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        self._set_price_oracle(new_oracle)
    }
    default fn support_market(&mut self, pool: AccountId) -> Result<()> {
        self._support_market(pool)
    }
    default fn support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._support_market_with_collateral_factor_mantissa(pool, collateral_factor_mantissa)
    }
    default fn set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_collateral_factor_mantissa(pool, new_collateral_factor_mantissa)
    }
    default fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._set_mint_guardian_paused(pool, paused)
    }
    default fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._set_borrow_guardian_paused(pool, paused)
    }
    default fn set_close_factor_mantissa(
        &mut self,
        new_close_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_close_factor_mantissa(new_close_factor_mantissa)
    }
    default fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_liquidation_incentive_mantissa(new_liquidation_incentive_mantissa)
    }
    default fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()> {
        self._set_borrow_cap(pool, new_cap)
    }
    default fn set_reserve_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._set_reserve_factor_mantissa(pool, new_reserve_factor_mantissa)
    }
    default fn reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
        self._reduce_reserves(pool, amount)
    }
    default fn sweep_token(&mut self, pool: AccountId, asset: AccountId) -> Result<()> {
        self._sweep_token(pool, asset)
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
        ControllerRef::set_price_oracle(&self._controller(), new_oracle).unwrap();
        Ok(())
    }
    default fn _support_market(&mut self, pool: AccountId) -> Result<()> {
        ControllerRef::support_market(&self._controller(), pool).unwrap();
        Ok(())
    }
    default fn _support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        ControllerRef::support_market_with_collateral_factor_mantissa(
            &self._controller(),
            pool,
            collateral_factor_mantissa,
        )
        .unwrap();
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
        )
        .unwrap();
        Ok(())
    }
    default fn _set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        ControllerRef::set_mint_guardian_paused(&self._controller(), pool, paused).unwrap();
        Ok(())
    }
    default fn _set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        ControllerRef::set_borrow_guardian_paused(&self._controller(), pool, paused).unwrap();
        Ok(())
    }
    default fn _set_close_factor_mantissa(
        &mut self,
        new_close_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        ControllerRef::set_close_factor_mantissa(&self._controller(), new_close_factor_mantissa)
            .unwrap();
        Ok(())
    }
    default fn _set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        ControllerRef::set_liquidation_incentive_mantissa(
            &self._controller(),
            new_liquidation_incentive_mantissa,
        )
        .unwrap();
        Ok(())
    }
    default fn _set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()> {
        ControllerRef::set_borrow_cap(&self._controller(), pool, new_cap).unwrap();
        Ok(())
    }
    default fn _set_reserve_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_reserve_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        PoolRef::set_reserve_factor_mantissa(&pool, new_reserve_factor_mantissa).unwrap();
        Ok(())
    }
    default fn _reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
        PoolRef::reduce_reserves(&pool, amount).unwrap();
        Ok(())
    }
    default fn _sweep_token(&mut self, pool: AccountId, asset: AccountId) -> Result<()> {
        PoolRef::sweep_token(&pool, asset).unwrap();
        Ok(())
    }
}
