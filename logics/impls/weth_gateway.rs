pub use crate::traits::weth_gateway::*;
use crate::traits::{
    pool::PoolRef,
    weth::*,
};
use ink::prelude::vec::Vec;
use openbrush::{
    contracts::{
        ownable::*,
        psp22::*,
    },
    traits::{
        AccountId,
        Balance,
        Storage,
        ZERO_ADDRESS,
    },
};
pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub weth: AccountId,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            weth: ZERO_ADDRESS.into(),
        }
    }
}

pub trait Internal {
    fn _safe_transfer_eth(&self, to: AccountId, value: Balance) -> Result<()>;
}

impl<T: Storage<Data> + Storage<ownable::Data>> Internal for T {
    default fn _safe_transfer_eth(&self, to: AccountId, value: Balance) -> Result<()> {
        let transfer_result = Self::env().transfer(to, value);
        if transfer_result.is_err() {
            return Err(WETHGatewayError::SafeETHTransferFailed)
        }
        Ok(())
    }
}

impl<T> WETHGateway for T
where
    T: Storage<Data> + Storage<ownable::Data>,
{
    default fn authorize_pool(&mut self, pool: AccountId) -> Result<()> {
        WETHRef::approve(&self.data::<Data>().weth, pool, u128::MAX)?;
        Ok(())
    }

    default fn deposit_eth(&mut self, pool: AccountId, on_behalf_of: AccountId) -> Result<()> {
        let deposit_value = Self::env().transferred_value();
        WETHRef::deposit_builder(&self.data::<Data>().weth)
            .transferred_value(deposit_value)
            .invoke()?;
        WETHRef::approve(&self.data::<Data>().weth, pool, deposit_value)?;
        PoolRef::mint_to(&pool, on_behalf_of, deposit_value)?;
        Ok(())
    }

    default fn withdraw_eth(
        &mut self,
        pool: AccountId,
        amount: Balance,
        to: AccountId,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let contract_address = Self::env().account_id();
        let user_balance: Balance = PoolRef::balance_of(&pool, caller);
        let mut amount_to_withdraw: Balance = amount;

        if amount == u128::MAX {
            amount_to_withdraw = user_balance;
        }

        PoolRef::transfer_from(
            &pool,
            caller,
            contract_address,
            amount_to_withdraw,
            Vec::<u8>::new(),
        )?;
        // PoolRef::redeem_underlying(&pool, amount_to_withdraw)?;
        // WETHRef::withdraw(&self.data::<Data>().weth, amount_to_withdraw)?;
        // self._safe_transfer_eth(to, amount_to_withdraw)
        Ok(())
    }

    default fn repay_eth(
        &mut self,
        pool: AccountId,
        amount: Balance,
        on_behalf_of: AccountId,
    ) -> Result<()> {
        let transferred_value = Self::env().transferred_value();
        let mut payback_amount = PoolRef::borrow_balance_stored(&pool, on_behalf_of);
        if amount < payback_amount {
            payback_amount = amount;
        }
        if transferred_value < payback_amount {
            return Err(WETHGatewayError::InsufficientPayback)
        }

        WETHRef::deposit_builder(&self.data::<Data>().weth)
            .transferred_value(payback_amount)
            .invoke()?;

        PoolRef::repay_borrow_behalf(&self.data::<Data>().weth, on_behalf_of, transferred_value)?;

        let caller = Self::env().caller();
        if transferred_value > payback_amount {
            self._safe_transfer_eth(caller, transferred_value - payback_amount)?;
        }
        Ok(())
    }

    default fn borrow_eth(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
        let caller = Self::env().caller();
        PoolRef::borrow(&pool, amount)?;
        WETHRef::withdraw(&self.data::<Data>().weth, amount)?;

        self._safe_transfer_eth(caller, amount)
    }

    default fn emergency_token_transfer(
        &mut self,
        token: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> Result<()> {
        PSP22Ref::transfer(&token, to, amount, Vec::<u8>::new())?;
        Ok(())
    }

    default fn emergency_ether_transfer(&mut self, to: AccountId, amount: Balance) -> Result<()> {
        self._safe_transfer_eth(to, amount)
    }

    default fn get_weth_address(&self) -> AccountId {
        self.data::<Data>().weth
    }
}
