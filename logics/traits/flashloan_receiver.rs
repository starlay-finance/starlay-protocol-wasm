use ink::prelude::vec::Vec;
use openbrush::traits::{
    AccountId,
    Balance,
};

#[openbrush::wrapper]
pub type FlashloanReceiverRef = dyn FlashloanReceiver;

#[openbrush::trait_definition]
pub trait FlashloanReceiver {
    /// Run FlashLoan action
    #[ink(message)]
    fn execute_operation(
        &self,
        assets: Vec<AccountId>,
        amounts: Vec<Balance>,
        premiums: Vec<Balance>,
        initiator: AccountId,
        params: Vec<u8>,
    ) -> bool;
}
