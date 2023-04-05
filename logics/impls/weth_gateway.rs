pub use crate::traits::weth_gateway::*;
use crate::traits::{
    pool::PoolRef,
    weth::*,
};
use ink::prelude::vec::Vec;
// use ink_env::call::{
//     CallBuilder,
//     FromAccountId,
// };
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
    default fn authorize_lending_pool(&mut self, lending_pool: AccountId) -> Result<()> {
        let approve_result = WETHRef::approve(&self.data::<Data>().weth, lending_pool, u128::MAX);
        if approve_result.is_err() {
            return Err(WETHGatewayError::from(approve_result.err().unwrap()))
        }
        Ok(())
    }

    default fn deposit_eth(
        &mut self,
        lending_pool: AccountId,
        on_behalf_of: AccountId,
        // referral_code: u16,
    ) -> Result<()> {
        let deposit_value = Self::env().transferred_value();
        // let weth_contract: WETHRef = FromAccountId::from_account_id(self.data::<Data>().weth);
        // WETH.deposit{value: msg.value}();
        // TODO: need to make payable call with transferred_value
        let deposit_result = WETHRef::deposit_builder(&self.data::<Data>().weth)
            .transferred_value(deposit_value)
            .invoke();
        if deposit_result.is_err() {
            return Err(WETHGatewayError::from(deposit_result.err().unwrap()))
        }
        // ILendingPool(lendingPool).deposit(address(WETH), msg.value, onBehalfOf, referralCode);
        let mint_result = PoolRef::mint_to(&lending_pool, on_behalf_of, deposit_value);
        if mint_result.is_err() {
            return Err(WETHGatewayError::from(mint_result.err().unwrap()))
        }

        Ok(())
    }

    default fn withdraw_eth(
        &mut self,
        lending_pool: AccountId,
        amount: Balance,
        to: AccountId,
    ) -> Result<()> {
        let caller = Self::env().caller();
        // ILToken lWETH = ILToken(ILendingPool(lendingPool).getReserveData(address(WETH)).lTokenAddress);
        let contract_address = Self::env().account_id();
        let user_balance: Balance = PoolRef::balance_of(&lending_pool, caller);
        let mut amount_to_withdraw: Balance = amount;

        if amount == u128::MAX {
            amount_to_withdraw = user_balance;
        }

        let transfer_result = PoolRef::transfer_from(
            &lending_pool,
            caller,
            contract_address,
            amount_to_withdraw,
            Vec::<u8>::new(),
        );
        if transfer_result.is_err() {
            return Err(WETHGatewayError::from(transfer_result.err().unwrap()))
        }
        // ILendingPool(lendingPool).withdraw(address(WETH), amountToWithdraw, address(this));
        let redeem_result = PoolRef::redeem(&lending_pool, amount_to_withdraw);
        if redeem_result.is_err() {
            return Err(WETHGatewayError::from(redeem_result.err().unwrap()))
        }
        let withdraw_result = WETHRef::withdraw(&self.data::<Data>().weth, amount_to_withdraw);
        if withdraw_result.is_err() {
            return Err(WETHGatewayError::from(withdraw_result.err().unwrap()))
        }
        self._safe_transfer_eth(to, amount_to_withdraw)
    }

    default fn repay_eth(
        &mut self,
        lending_pool: AccountId,
        amount: Balance,
        // rate_mode: u128,
        on_behalf_of: AccountId,
    ) -> Result<()> {
        let transferred_value = Self::env().transferred_value();
        let mut payback_amount = PoolRef::borrow_balance_stored(&lending_pool, on_behalf_of);
        if amount < payback_amount {
            payback_amount = amount;
        }
        if transferred_value < payback_amount {
            return Err(WETHGatewayError::InsufficientPayback)
        }

        let deposit_result = WETHRef::deposit_builder(&self.data::<Data>().weth)
            .transferred_value(payback_amount)
            .invoke();
        if deposit_result.is_err() {
            return Err(WETHGatewayError::from(deposit_result.err().unwrap()))
        }

        let repay_result = PoolRef::repay_borrow_behalf(
            &self.data::<Data>().weth,
            on_behalf_of,
            transferred_value,
        );
        if repay_result.is_err() {
            return Err(WETHGatewayError::from(repay_result.err().unwrap()))
        }

        let caller = Self::env().caller();
        if transferred_value > payback_amount {
            let transfer_result =
                self._safe_transfer_eth(caller, transferred_value - payback_amount);
            if transfer_result.is_err() {
                return transfer_result
            }
        }
        Ok(())
    }

    default fn borrow_eth(
        &mut self,
        lending_pool: AccountId,
        amount: Balance,
        interes_rate_mode: u128,
        referral_code: u16,
    ) -> Result<()> {
        let caller = Self::env().caller();
        // ILendingPool(lendingPool).borrow(
        //     address(WETH),
        //     amount,
        //     interesRateMode,
        //     referralCode,
        //     msg.sender
        //   );
        let withdraw_result = WETHRef::withdraw(&self.data::<Data>().weth, amount);
        if withdraw_result.is_err() {
            return Err(WETHGatewayError::from(withdraw_result.err().unwrap()))
        }
        self._safe_transfer_eth(caller, amount)
    }

    default fn emergency_token_transfer(
        &mut self,
        token: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> Result<()> {
        let transfer_result = PSP22Ref::transfer(&token, to, amount, Vec::<u8>::new());
        if transfer_result.is_err() {
            return Err(WETHGatewayError::from(transfer_result.err().unwrap()))
        }
        Ok(())
    }

    default fn emergency_ether_transfer(&mut self, to: AccountId, amount: Balance) -> Result<()> {
        self._safe_transfer_eth(to, amount)
    }

    default fn get_weth_address(&self) -> AccountId {
        self.data::<Data>().weth
    }
}
