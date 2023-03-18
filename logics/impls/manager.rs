pub use crate::traits::manager::*;
use crate::traits::pool::PoolRef;
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
    fn _reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()>;
}

impl<T: Storage<Data>> Manager for T {
    default fn controller(&self) -> AccountId {
        self._controller()
    }
    default fn set_controller(&mut self, id: AccountId) -> Result<()> {
        self._set_controller(id)
    }
    default fn reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
        self._reduce_reserves(pool, amount)
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
    default fn _reduce_reserves(&mut self, pool: AccountId, amount: Balance) -> Result<()> {
        PoolRef::reduce_reserves(&pool, amount).unwrap();
        Ok(())
    }
}
