#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use logics::impls::pool::*;
    use openbrush::{
        contracts::psp22::{
            extensions::metadata::{
                self,
                PSP22MetadataRef,
            },
            psp22,
        },
        traits::{
            Storage,
            String,
        },
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct PoolContract {
        #[storage_field]
        pool: Data,
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl Pool for PoolContract {}

    impl psp22::PSP22 for PoolContract {}

    impl metadata::PSP22Metadata for PoolContract {}

    impl PoolContract {
        #[ink(constructor)]
        pub fn new(
            underlying: AccountId,
            controller: AccountId,
            name: String,
            symbol: String,
            decimals: u8,
        ) -> Self {
            let mut instance = Self::default();
            instance._initialize(underlying, controller, name, symbol, decimals);
            instance
        }

        #[ink(constructor)]
        pub fn new_from_asset(underlying: AccountId, controller: AccountId) -> Self {
            let base_name = PSP22MetadataRef::token_name(&underlying);
            let base_symbol = PSP22MetadataRef::token_symbol(&underlying);
            let decimals = PSP22MetadataRef::token_decimals(&underlying);

            let mut name = "Starlay ".as_bytes().to_vec();
            name.append(&mut base_name.unwrap());
            let mut symbol = "s".as_bytes().to_vec();
            symbol.append(&mut base_symbol.unwrap());

            let mut instance = Self::default();
            instance._initialize(underlying, controller, name, symbol, decimals);
            instance
        }

        fn _initialize(
            &mut self,
            underlying: AccountId,
            controller: AccountId,
            name: String,
            symbol: String,
            decimals: u8,
        ) {
            self.pool.underlying = underlying;
            self.pool.controller = controller;
            self.metadata.name = Some(name);
            self.metadata.symbol = Some(symbol);
            self.metadata.decimals = decimals;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{
                self,
                DefaultAccounts,
            },
            DefaultEnvironment,
        };

        fn default_accounts() -> DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }
        fn set_caller(id: AccountId) {
            test::set_caller::<DefaultEnvironment>(id);
        }

        #[ink::test]
        fn new_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let underlying = AccountId::from([0x01; 32]);
            let controller = AccountId::from([0x02; 32]);
            let contract = PoolContract::new(
                underlying,
                controller,
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );
            assert_eq!(contract.underlying(), underlying);
        }
    }
}
