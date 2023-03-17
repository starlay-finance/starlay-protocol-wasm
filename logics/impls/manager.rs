pub use crate::traits::manager::*;
use openbrush::traits::{
    AccountId,
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
}

impl<T: Storage<Data>> Manager for T {
    default fn controller(&self) -> AccountId {
        self._controller()
    }
    default fn set_controller(&mut self, id: AccountId) -> Result<()> {
        self._set_controller(id)
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
}
