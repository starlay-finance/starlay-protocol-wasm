use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp22::PSP22Ref,
    traits::{
        AccountId,
        Balance,
        Storage,
        ZERO_ADDRESS,
    },
};

pub use crate::traits::flashloan_receiver::*;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub flashloan_gateway: AccountId,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            flashloan_gateway: ZERO_ADDRESS.into(),
        }
    }
}

pub trait Internal {
    fn _initialize(&mut self, flashloan_gateway: AccountId);
}

impl<T: Storage<Data>> Internal for T {
    default fn _initialize(&mut self, flashloan_gateway: AccountId) {
        self.data().flashloan_gateway = flashloan_gateway;
    }
}

impl<T: Storage<Data>> FlashloanReceiver for T {
    default fn execute_operation(
        &self,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        premiums: Vec<Balance>,
        _initiator: AccountId,
        _params: Vec<u8>,
    ) -> bool {
        let contract_addr = Self::env().account_id();
        let gateway = self.data().flashloan_gateway;
        for index in 1..assets.len() {
            let current_asset = assets[index];
            let balance = PSP22Ref::balance_of(&current_asset, contract_addr);

            let amount_to_return = amounts[index] + premiums[index];

            if balance < amount_to_return {
                return false
            }

            let approve_result = PSP22Ref::approve(&current_asset, gateway, amount_to_return);
            if approve_result.is_err() {
                return false
            }
        }
        true
    }
}
