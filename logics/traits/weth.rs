use openbrush::{
    contracts::psp22::extensions::metadata::*,
    traits::Balance,
};

#[openbrush::wrapper]
pub type WETHRef = dyn WETH + PSP22 + PSP22Metadata;

#[openbrush::trait_definition]
pub trait WETH: PSP22 + PSP22Metadata {
    #[ink(message, payable)]
    fn deposit(&mut self) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn withdraw(&mut self, value: Balance) -> Result<(), PSP22Error>;
}
