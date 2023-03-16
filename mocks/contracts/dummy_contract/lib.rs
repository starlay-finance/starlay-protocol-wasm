#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    Ng,
}

#[openbrush::contract]
pub mod dummy_contract {
    use super::Error;

    #[ink(storage)]
    #[derive(Default)]
    pub struct DummyContract {}

    impl DummyContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
        #[ink(message)]
        pub fn result_ok(&mut self) -> core::result::Result<(), Error> {
            Ok(())
        }

        #[ink(message)]
        pub fn result_ng(&mut self) -> core::result::Result<(), Error> {
            Err(Error::Ng)
        }

        #[ink(message)]
        pub fn result_panic(&mut self) -> core::result::Result<(), Error> {
            panic!("panic");
            Ok(())
        }
    }
}
