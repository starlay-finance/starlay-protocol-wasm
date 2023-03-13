#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use logics::impls::pool::{
        Internal,
        *,
    };
    use openbrush::{
        contracts::psp22::{
            extensions::metadata::{
                self,
                PSP22MetadataRef,
            },
            psp22,
        },
        traits::{
            AccountIdExt,
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

    #[ink(event)]
    pub struct Mint {
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    }
    #[ink(event)]
    pub struct Redeem {
        redeemer: AccountId,
        redeem_amount: Balance,
        redeem_tokens: Balance,
    }
    #[ink(event)]
    pub struct Borrow {
        borrower: AccountId,
        borrow_amount: Balance,
        account_borrows: Balance,
        total_borrows: Balance,
    }

    impl Pool for PoolContract {}
    impl Internal for PoolContract {
        fn _emit_mint_event(&self, minter: AccountId, mint_amount: Balance, mint_tokens: Balance) {
            self.env().emit_event(Mint {
                minter,
                mint_amount,
                mint_tokens,
            })
        }
        fn _emit_redeem_event(
            &self,
            redeemer: AccountId,
            redeem_amount: Balance,
            redeem_tokens: Balance,
        ) {
            self.env().emit_event(Redeem {
                redeemer,
                redeem_amount,
                redeem_tokens,
            })
        }
        fn _emit_borrow_event(
            &self,
            borrower: AccountId,
            borrow_amount: Balance,
            account_borrows: Balance,
            total_borrows: Balance,
        ) {
            self.env().emit_event(Borrow {
                borrower,
                borrow_amount,
                account_borrows,
                total_borrows,
            })
        }
    }

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
            if underlying.is_zero() {
                panic!("underlying is zero address");
            }
            if controller.is_zero() {
                panic!("controller is zero address");
            }
            let mut instance = Self::default();
            instance._initialize(underlying, controller, name, symbol, decimals);
            instance
        }

        #[ink(constructor)]
        pub fn new_from_asset(underlying: AccountId, controller: AccountId) -> Self {
            if underlying.is_zero() {
                panic!("underlying is zero address");
            }
            if controller.is_zero() {
                panic!("controller is zero address");
            }

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
        use openbrush::traits::ZERO_ADDRESS;

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
            assert_eq!(contract.controller(), controller);
            assert_eq!(contract.total_borrows(), 0);
        }

        #[ink::test]
        #[should_panic(expected = "underlying is zero address")]
        fn new_works_when_underlying_is_zero_address() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let controller = AccountId::from([0x02; 32]);
            PoolContract::new(
                ZERO_ADDRESS.into(),
                controller,
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );
        }

        #[ink::test]
        #[should_panic(expected = "controller is zero address")]
        fn new_works_when_controller_is_zero_address() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let underlying = AccountId::from([0x01; 32]);
            PoolContract::new(
                underlying,
                ZERO_ADDRESS.into(),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );
        }
    }
}
