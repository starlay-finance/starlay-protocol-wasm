use crate::traits::weth::*;
pub use crate::traits::weth_gateway::*;
// use ink_env::call::{FromAccountId, CallBuilder};
use ink::codegen::{Env::call::{FromAccountId, CallBuilder}};
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
    fn _safe_transfer_eth(&self, to: AccountId, value: Balance) -> Result<(), WETHGatewayError>;
}

impl<T: Storage<Data> + Storage<ownable::Data>> Internal for T {
    default fn _safe_transfer_eth(
        &self,
        to: AccountId,
        value: Balance,
    ) -> Result<(), WETHGatewayError> {
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
    default fn authorize_lending_pool(
        &mut self,
        lending_pool: AccountId,
    ) -> Result<(), WETHGatewayError> {
        let approve_result = WETHRef::approve(
            &self.data::<Data>().weth,
            lending_pool,
            (-1_i128).try_into().unwrap(),
        );
        if approve_result.is_err() {
            return Err(WETHGatewayError::from(approve_result.err().unwrap()))
        }
        Ok(())
    }

    default fn deposit_eth(
        &mut self,
        lending_pool: AccountId,
        on_behalf_of: AccountId,
        referral_code: u16,
    ) {
        let deposit_value = Self::env().transferred_value();
        // let weth_contract: FromAccountId<WETH> = FromAccountId::from_account_id(self.data::<Data>().weth);
        // let result = CallBuilder::using(&mut weth_contract)
        //     .gas_limit(500000)
        //     .with_value(deposit_value)
        //     .build()
        //     .deposit();
    }

    default fn withdraw_eth(
        &mut self,
        lending_pool: AccountId,
        amount: Balance,
        on_behalf_of: AccountId,
    ) {
    }

    default fn repay_eth(
        &mut self,
        lending_pool: AccountId,
        amount: Balance,
        rate_mode: u128,
        on_behalf_of: AccountId,
    ) {
    }

    default fn borrow_eth(
        &mut self,
        lending_pool: AccountId,
        amount: Balance,
        interes_rate_mode: u128,
        referral_code: u16,
    ) {
    }

    default fn emergency_token_transfer(
        &mut self,
        token: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), WETHGatewayError> {
        let transfer_result = PSP22Ref::transfer(&token, to, amount, Vec::<u8>::new());
        if transfer_result.is_err() {
            return Err(WETHGatewayError::from(transfer_result.err().unwrap()))
        }
        Ok(())
    }

    default fn emergency_ether_transfer(
        &mut self,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), WETHGatewayError> {
        self._safe_transfer_eth(to, amount)
    }

    default fn get_weth_address(&self) -> AccountId {
        self.data::<Data>().weth
    }
}
