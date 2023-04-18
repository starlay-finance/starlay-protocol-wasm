use openbrush::{
    contracts::{
        ownable::*,
        psp22::extensions::metadata::*,
    },
    modifiers,
    traits::{
        AccountId,
        Balance,
    },
};

pub use super::pool::Error as PoolError;

#[openbrush::wrapper]
pub type WETHGatewayRef = dyn WETHGateway + Ownable;

#[openbrush::trait_definition]
pub trait WETHGateway: Ownable {
    #[ink(message)]
    #[modifiers(only_owner)]
    fn authorize_pool(&mut self, pool: AccountId) -> Result<()>;

    #[ink(message, payable)]
    fn deposit_eth(
        &mut self,
        pool: AccountId,
        on_behalf_of: AccountId,
        // referral_code: u16,
    ) -> Result<()>;

    #[ink(message)]
    fn withdraw_eth(
        &mut self,
        pool: AccountId,
        amount: Balance,
        on_behalf_of: AccountId,
    ) -> Result<()>;

    #[ink(message, payable)]
    fn repay_eth(
        &mut self,
        pool: AccountId,
        amount: Balance,
        // rate_mode: u128,
        on_behalf_of: AccountId,
    ) -> Result<()>;

    #[ink(message)]
    fn borrow_eth(
        &mut self,
        pool: AccountId,
        amount: Balance,
        // interes_rate_mode: u128,
        // referral_code: u16,
    ) -> Result<()>;

    #[ink(message)]
    #[modifiers(only_owner)]
    fn emergency_token_transfer(
        &mut self,
        token: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> Result<()>;

    #[ink(message)]
    #[modifiers(only_owner)]
    fn emergency_ether_transfer(&mut self, to: AccountId, amount: Balance) -> Result<()>;

    #[ink(message)]
    fn get_weth_address(&self) -> AccountId;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum WETHGatewayError {
    SafeETHTransferFailed,
    InsufficientPayback,
    Pool(PoolError),
    PSP22(PSP22Error),
}

impl From<PSP22Error> for WETHGatewayError {
    fn from(error: PSP22Error) -> Self {
        WETHGatewayError::PSP22(error)
    }
}

impl From<PoolError> for WETHGatewayError {
    fn from(error: PoolError) -> Self {
        WETHGatewayError::Pool(error)
    }
}

pub type Result<T> = core::result::Result<T, WETHGatewayError>;