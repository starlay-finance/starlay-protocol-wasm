#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod token {
    use logics::traits::{
        controller::ControllerRef,
        pool::PoolRef,
    };
    use openbrush::{
        contracts::traits::psp22::{
            extensions::mintable::PSP22MintableRef,
            PSP22Error,
        },
        traits::Storage,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Faucet {}

    impl Faucet {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[ink(message)]
        pub fn mint(
            &self,
            asset: AccountId,
            amount: Balance,
            account: Option<AccountId>,
        ) -> Result<(), PSP22Error> {
            self._mint(&asset, amount, account)
        }

        #[ink(message)]
        pub fn mint_underlying(
            &self,
            pool: AccountId,
            amount: Balance,
            account: Option<AccountId>,
        ) -> Result<(), PSP22Error> {
            self._mint_underlying(&pool, amount, account)
        }

        #[ink(message)]
        pub fn mint_underlying_all(
            &self,
            controller: AccountId,
            amount: Balance,
            account: Option<AccountId>,
        ) -> Result<(), PSP22Error> {
            let pools = ControllerRef::markets(&controller);
            for pool in pools.iter() {
                self._mint_underlying(pool, amount, account)?;
            }
            Ok(())
        }

        fn _mint_underlying(
            &self,
            pool: &AccountId,
            amount: Balance,
            account: Option<AccountId>,
        ) -> Result<(), PSP22Error> {
            let underlying = PoolRef::underlying(pool);
            self._mint(&underlying, amount, account)
        }

        fn _mint(
            &self,
            asset: &AccountId,
            amount: Balance,
            account: Option<AccountId>,
        ) -> Result<(), PSP22Error> {
            PSP22MintableRef::mint(asset, account.unwrap_or(Self::env().caller()), amount)
        }
    }
}
