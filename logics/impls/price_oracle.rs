pub use crate::traits::price_oracle::*;
use openbrush::traits::{
    AccountId,
    Storage,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    // TODO
}

pub trait Internal {
    fn _get_price(&self, asset: AccountId) -> u128;
}

impl<T: Storage<Data>> PriceOracle for T {
    default fn get_price(&self, asset: AccountId) -> u128 {
        self._get_price(asset)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _get_price(&self, _asset: AccountId) -> u128 {
        // TODO
        0
    }
}
