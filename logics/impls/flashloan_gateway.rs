// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::traits::flashloan_gateway::*;
use crate::traits::{
    controller::ControllerRef,
    flashloan_receiver::FlashloanReceiverRef,
    pool::PoolRef,
};
use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp22::PSP22Ref,
    traits::{
        AccountId,
        Balance,
        Storage,
    },
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug, Default)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// Flashloan Fee in percentage * 100.
    /// Default value is 9 = 0.09%
    pub flashloan_premium_total: u128,
    /// AccountId of Controller managing Flashloan Gateway
    pub controller: Option<AccountId>,
}

pub trait Internal {
    fn _initialize(&mut self, controller: AccountId);

    // View function
    fn _flashloan_premium_total(&self) -> u128;
    fn _controller(&self) -> Option<AccountId>;
    // events
    fn _emit_flashloan_event(
        &self,
        target: AccountId,
        initiator: AccountId,
        asset: AccountId,
        amount: Balance,
        premium: Balance,
    );
}

impl<T: Storage<Data>> FlashloanGateway for T {
    default fn flashloan(
        &self,
        receiver_address: AccountId,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        mods: Vec<u8>,
        _on_behalf_of: AccountId,
        params: Vec<u8>,
    ) -> Result<()> {
        if assets.len() != amounts.len() {
            return Err(Error::InconsistentFlashloanParams)
        }

        let mut lp_token_addresses: Vec<AccountId> = Default::default();
        let mut premiums: Vec<Balance> = Default::default();

        let controller = self._controller();
        if controller.is_none() {
            return Err(Error::ControllerIsNotSet)
        }
        let _controller = controller.unwrap();

        let flashloan_premium_total = self._flashloan_premium_total();
        for index in 0..assets.len() {
            let market = ControllerRef::market_of_underlying(&_controller, assets[index]);
            if market.is_none() {
                return Err(Error::MarketNotListed)
            }
            lp_token_addresses.push(market.unwrap());
            let premium: u128 = amounts[index] * flashloan_premium_total / 10000;
            premiums.push(premium);

            PoolRef::transfer_underlying(
                &lp_token_addresses[index],
                receiver_address,
                amounts[index],
            )?;
        }

        let caller = Self::env().caller();
        let operation_result = FlashloanReceiverRef::execute_operation(
            &receiver_address,
            assets.clone(),
            amounts.clone(),
            premiums.clone(),
            caller,
            params.clone(),
        );

        if !operation_result {
            return Err(Error::InvalidFlashloanExecutorReturn)
        }

        for index in 0..assets.len() {
            let current_asset = assets[index];
            let current_amount = amounts[index];
            let current_premium = premiums[index];
            let current_lp_token = lp_token_addresses[index];
            let current_amount_plus_premium = current_amount + current_premium;

            if mods[index] == FlashLoanType::None as u8 {
                PoolRef::accrue_interest(&current_lp_token)?;

                PSP22Ref::transfer_from(
                    &current_asset,
                    receiver_address,
                    current_lp_token,
                    current_amount_plus_premium,
                    Vec::<u8>::new(),
                )?;
            } else {
                PoolRef::borrow_for_flashloan(&current_lp_token, caller, current_amount)?;
            }

            self._emit_flashloan_event(
                receiver_address,
                caller,
                current_asset,
                current_amount,
                current_premium,
            );
        }

        Ok(())
    }

    default fn flashloan_premium_total(&self) -> u128 {
        self._flashloan_premium_total()
    }

    default fn controller(&self) -> Option<AccountId> {
        self._controller()
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _initialize(&mut self, controller: AccountId) {
        self.data::<Data>().flashloan_premium_total = 9;
        self.data::<Data>().controller = Some(controller);
    }

    default fn _flashloan_premium_total(&self) -> u128 {
        self.data::<Data>().flashloan_premium_total
    }

    default fn _controller(&self) -> Option<AccountId> {
        self.data::<Data>().controller
    }

    default fn _emit_flashloan_event(
        &self,
        _target: AccountId,
        _initiator: AccountId,
        _asset: AccountId,
        _amount: Balance,
        _premium: Balance,
    ) {
    }
}
