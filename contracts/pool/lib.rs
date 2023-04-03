#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use ink::{
        codegen::{
            EmitEvent,
            Env,
        },
        prelude::vec::Vec,
    };
    use logics::{
        impls::pool::{
            Internal,
            *,
        },
        traits::types::WrappedU256,
    };
    use openbrush::{
        contracts::psp22::{
            extensions::metadata::{
                self,
                PSP22MetadataRef,
            },
            psp22,
            PSP22Error,
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
    pub struct AccrueInterest {
        interest_accumulated: Balance,
        borrow_index: WrappedU256,
        total_borrows: Balance,
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
    #[ink(event)]
    pub struct RepayBorrow {
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        account_borrows: Balance,
        total_borrows: Balance,
    }
    #[ink(event)]
    pub struct LiquidateBorrow {
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        token_collateral: AccountId,
        seize_tokens: Balance,
    }
    #[ink(event)]
    pub struct ReservesAdded {
        benefactor: AccountId,
        add_amount: Balance,
        new_total_reserves: Balance,
    }
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    impl Pool for PoolContract {
        #[ink(message)]
        fn repay_borrow_behalf(
            &mut self,
            _borrower: AccountId,
            _repay_amount: Balance,
        ) -> Result<()> {
            Err(Error::NotImplemented)
        }

        #[ink(message)]
        fn set_controller(&mut self, _new_controller: AccountId) -> Result<()> {
            Err(Error::NotImplemented)
        }

        #[ink(message)]
        fn add_reserves(&mut self, _amount: Balance) -> Result<()> {
            Err(Error::NotImplemented)
        }

        #[ink(message)]
        fn set_interest_rate_model(&mut self, _new_interest_rate_model: AccountId) -> Result<()> {
            Err(Error::NotImplemented)
        }
    }
    impl Internal for PoolContract {
        fn _emit_accrue_interest_event(
            &self,
            interest_accumulated: Balance,
            borrow_index: WrappedU256,
            total_borrows: Balance,
        ) {
            self.env().emit_event(AccrueInterest {
                interest_accumulated,
                borrow_index,
                total_borrows,
            })
        }
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
        fn _emit_repay_borrow_event(
            &self,
            payer: AccountId,
            borrower: AccountId,
            repay_amount: Balance,
            account_borrows: Balance,
            total_borrows: Balance,
        ) {
            self.env().emit_event(RepayBorrow {
                payer,
                borrower,
                repay_amount,
                account_borrows,
                total_borrows,
            })
        }
        fn _emit_liquidate_borrow_event(
            &self,
            liquidator: AccountId,
            borrower: AccountId,
            repay_amount: Balance,
            token_collateral: AccountId,
            seize_tokens: Balance,
        ) {
            self.env().emit_event(LiquidateBorrow {
                liquidator,
                borrower,
                repay_amount,
                token_collateral,
                seize_tokens,
            })
        }
        fn _emit_reserves_added_event(
            &self,
            benefactor: AccountId,
            add_amount: Balance,
            new_total_reserves: Balance,
        ) {
            self.env().emit_event(ReservesAdded {
                benefactor,
                add_amount,
                new_total_reserves,
            })
        }
    }

    impl psp22::PSP22 for PoolContract {
        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> core::result::Result<(), PSP22Error> {
            let caller = self.env().caller();
            self._transfer_tokens(caller, caller, to, value, data)
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> core::result::Result<(), PSP22Error> {
            self._transfer_tokens(self.env().caller(), from, to, value, data)
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            Internal::_balance_of(self, &owner)
        }

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            Internal::_total_supply(self)
        }
    }
    impl psp22::Internal for PoolContract {
        fn _emit_transfer_event(
            &self,
            from: Option<AccountId>,
            to: Option<AccountId>,
            value: Balance,
        ) {
            self.env().emit_event(Transfer { from, to, value });
        }

        fn _emit_approval_event(&self, owner: AccountId, spender: AccountId, value: Balance) {
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
        }
    }

    impl metadata::PSP22Metadata for PoolContract {}

    impl PoolContract {
        #[ink(constructor)]
        pub fn new(
            underlying: AccountId,
            controller: AccountId,
            rate_model: AccountId,
            initial_exchange_rate_mantissa: WrappedU256,
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
            instance._initialize(
                underlying,
                controller,
                Self::env().caller(),
                rate_model,
                initial_exchange_rate_mantissa,
                name,
                symbol,
                decimals,
            );
            instance
        }

        #[ink(constructor)]
        pub fn new_from_asset(
            underlying: AccountId,
            controller: AccountId,
            rate_model: AccountId,
            initial_exchange_rate_mantissa: WrappedU256,
        ) -> Self {
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
            instance._initialize(
                underlying,
                controller,
                Self::env().caller(),
                rate_model,
                initial_exchange_rate_mantissa,
                name,
                symbol,
                decimals,
            );
            instance
        }

        fn _initialize(
            &mut self,
            underlying: AccountId,
            controller: AccountId,
            manager: AccountId,
            rate_model: AccountId,
            initial_exchange_rate_mantissa: WrappedU256,
            name: String,
            symbol: String,
            decimals: u8,
        ) {
            self.pool.underlying = underlying;
            self.pool.controller = controller;
            self.pool.manager = manager;
            self.pool.rate_model = rate_model;
            self.pool.initial_exchange_rate_mantissa = initial_exchange_rate_mantissa;
            self.pool.accrual_block_timestamp = Self::env().block_timestamp();
            self.metadata.name = Some(name);
            self.metadata.symbol = Some(symbol);
            self.metadata.decimals = decimals;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::{
            env::{
                test::{
                    self,
                    DefaultAccounts,
                },
                DefaultEnvironment,
            },
            prelude::vec::Vec,
        };
        use logics::{
            impls::{
                exp_no_err::exp_scale,
                pool::Pool,
            },
            traits::types::WrappedU256,
        };
        use openbrush::{
            contracts::psp22::PSP22,
            traits::ZERO_ADDRESS,
        };
        use primitive_types::U256;
        use std::ops::{
            Add,
            Div,
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
            let rate_model = AccountId::from([0x03; 32]);
            let initial_exchange_rate_mantissa = WrappedU256::from(exp_scale());
            let contract = PoolContract::new(
                underlying,
                controller,
                rate_model,
                initial_exchange_rate_mantissa,
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );
            assert_eq!(contract.underlying(), underlying);
            assert_eq!(contract.controller(), controller);
            assert_eq!(contract.manager(), accounts.bob);
            assert_eq!(
                contract.initial_exchange_rate_mantissa(),
                initial_exchange_rate_mantissa
            );
            assert_eq!(
                contract.reserve_factor_mantissa(),
                WrappedU256::from(U256::from(0))
            );
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
                ZERO_ADDRESS.into(),
                WrappedU256::from(U256::from(0)),
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
                ZERO_ADDRESS.into(),
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );
        }

        #[ink::test]
        #[should_panic(
            expected = "not implemented: off-chain environment does not support contract invocation"
        )]
        fn transfer_works_overridden() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            contract.transfer(accounts.charlie, 0, Vec::new()).unwrap();
        }

        #[ink::test]
        #[should_panic(
            expected = "not implemented: off-chain environment does not support contract invocation"
        )]
        fn transfer_from_works_overridden() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            contract
                .transfer_from(accounts.bob, accounts.charlie, 0, Vec::new())
                .unwrap();
        }

        #[ink::test]
        #[should_panic(
            expected = "not implemented: off-chain environment does not support contract invocation"
        )]
        fn redeem_underlying_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            contract.redeem_underlying(0).unwrap();
        }

        #[ink::test]
        fn repay_borrow_behalf_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            assert_eq!(
                contract
                    .repay_borrow_behalf(accounts.charlie, 0)
                    .unwrap_err(),
                Error::NotImplemented
            )
        }

        #[ink::test]
        fn set_controller_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            assert_eq!(
                contract.set_controller(dummy_id).unwrap_err(),
                Error::NotImplemented
            )
        }

        #[ink::test]
        fn add_reserves_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            assert_eq!(contract.add_reserves(0).unwrap_err(), Error::NotImplemented)
        }

        #[ink::test]
        fn set_interest_rate_model_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            assert_eq!(
                contract.set_interest_rate_model(dummy_id).unwrap_err(),
                Error::NotImplemented
            )
        }

        #[ink::test]
        fn set_reserve_factor_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );
            assert_eq!(contract.reserve_factor_mantissa(), WrappedU256::from(0));
            let half_exp_scale = exp_scale().div(2);
            assert!(contract
                .set_reserve_factor_mantissa(WrappedU256::from(half_exp_scale))
                .is_ok());
            assert_eq!(
                contract.reserve_factor_mantissa(),
                WrappedU256::from(half_exp_scale)
            );
            let over_exp_scale = exp_scale().add(1);
            assert_eq!(
                contract
                    .set_reserve_factor_mantissa(WrappedU256::from(over_exp_scale))
                    .unwrap_err(),
                Error::SetReserveFactorBoundsCheck
            );
        }

        #[ink::test]
        fn assert_manager_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            set_caller(accounts.charlie);
            let admin_funcs: Vec<Result<()>> = vec![
                contract.reduce_reserves(100),
                contract.sweep_token(dummy_id),
                contract.set_reserve_factor_mantissa(WrappedU256::from(0)),
            ];
            for func in admin_funcs {
                assert_eq!(func.unwrap_err(), Error::CallerIsNotManager);
            }
        }
    }
}
