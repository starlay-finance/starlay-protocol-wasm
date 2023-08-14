// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

/// Definition of Pool Contract
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

    /// Contract's Storage
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

    /// Event: Execute `Mint`
    #[ink(event)]
    pub struct Mint {
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    }
    /// Event: Execute `Redeem`
    #[ink(event)]
    pub struct Redeem {
        redeemer: AccountId,
        redeem_amount: Balance,
    }
    /// Event: Execute `Borrow`
    #[ink(event)]
    pub struct Borrow {
        borrower: AccountId,
        borrow_amount: Balance,
        account_borrows: Balance,
        total_borrows: Balance,
    }
    /// Event: Execute `RepayBorrow`
    #[ink(event)]
    pub struct RepayBorrow {
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        account_borrows: Balance,
        total_borrows: Balance,
    }
    /// Event: Execute `LiquidateBorrow`
    #[ink(event)]
    pub struct LiquidateBorrow {
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        token_collateral: AccountId,
        seize_tokens: Balance,
    }
    /// Event: Adding to Reserves
    #[ink(event)]
    pub struct ReservesAdded {
        benefactor: AccountId,
        add_amount: Balance,
        new_total_reserves: Balance,
    }

    /// Event: Transfer Pool Token
    ///
    /// NOTE: Use event emitter included in PSP22 Interface
    /// [PSP22 | Brushfam](https://learn.brushfam.io/docs/OpenBrush/smart-contracts/PSP22/)
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }
    /// Event: Allowance of a spender for an owner is set
    ///
    /// NOTE: Use event emitter included in PSP22 Interface
    /// [PSP22 | Brushfam](https://learn.brushfam.io/docs/OpenBrush/smart-contracts/PSP22/)
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// Event: Delegation Allowance for Borrowing is changed
    #[ink(event)]
    pub struct DelegateApproval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        delegatee: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct ReserveUsedAsCollateralEnabled {
        #[ink(topic)]
        user: AccountId,
    }

    #[ink(event)]
    pub struct ReserveUsedAsCollateralDisabled {
        #[ink(topic)]
        user: AccountId,
    }

    impl Pool for PoolContract {
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
        fn _emit_mint_event(&self, minter: AccountId, mint_amount: Balance, mint_tokens: Balance) {
            self.env().emit_event(Mint {
                minter,
                mint_amount,
                mint_tokens,
            })
        }
        fn _emit_redeem_event(&self, redeemer: AccountId, redeem_amount: Balance) {
            self.env().emit_event(Redeem {
                redeemer,
                redeem_amount,
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

        fn _emit_delegate_approval_event(
            &self,
            owner: AccountId,
            delegatee: AccountId,
            amount: Balance,
        ) {
            self.env().emit_event(DelegateApproval {
                owner,
                delegatee,
                amount,
            })
        }

        fn _emit_reserve_used_as_collateral_enabled_event(&self, user: AccountId) {
            self.env()
                .emit_event(ReserveUsedAsCollateralEnabled { user })
        }

        fn _emit_reserve_used_as_collateral_disabled_event(&self, user: AccountId) {
            self.env()
                .emit_event(ReserveUsedAsCollateralDisabled { user })
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

    #[allow(clippy::too_many_arguments)]
    impl PoolContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(
            incentives_controller: AccountId,
            underlying: AccountId,
            controller: AccountId,
            rate_model: AccountId,
            initial_exchange_rate_mantissa: WrappedU256,
            liquidation_threshold: u128,
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
                incentives_controller,
                underlying,
                controller,
                Self::env().caller(),
                rate_model,
                initial_exchange_rate_mantissa,
                liquidation_threshold,
                name,
                symbol,
                decimals,
            );
            instance
        }

        /// Generate this contract
        #[ink(constructor)]
        pub fn new_from_asset(
            incentives_controller: AccountId,
            underlying: AccountId,
            controller: AccountId,
            rate_model: AccountId,
            initial_exchange_rate_mantissa: WrappedU256,
            liquidation_threshold: u128,
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

            let name = String::from("Starlay ") + &base_name.unwrap();
            let symbol = String::from("s") + &base_symbol.unwrap();

            let mut instance = Self::default();
            instance._initialize(
                incentives_controller,
                underlying,
                controller,
                Self::env().caller(),
                rate_model,
                initial_exchange_rate_mantissa,
                liquidation_threshold,
                name,
                symbol,
                decimals,
            );
            instance
        }

        #[allow(clippy::too_many_arguments)]
        fn _initialize(
            &mut self,
            incentives_controller: AccountId,
            underlying: AccountId,
            controller: AccountId,
            manager: AccountId,
            rate_model: AccountId,
            initial_exchange_rate_mantissa: WrappedU256,
            liquidation_threshold: u128,
            name: String,
            symbol: String,
            decimals: u8,
        ) {
            self.pool.incentives_controller = Some(incentives_controller);
            self.pool.underlying = Some(underlying);
            self.pool.controller = Some(controller);
            self.pool.manager = Some(manager);
            self.pool.rate_model = Some(rate_model);
            self.pool.initial_exchange_rate_mantissa = initial_exchange_rate_mantissa;
            self.pool.liquidation_threshold = liquidation_threshold;
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
        use openbrush::contracts::psp22::PSP22;
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
            let incentives_controller = AccountId::from([0x04; 32]);
            let initial_exchange_rate_mantissa = WrappedU256::from(exp_scale());
            let liquidation_threshold = 10000;
            let contract = PoolContract::new(
                incentives_controller,
                underlying,
                controller,
                rate_model,
                initial_exchange_rate_mantissa,
                liquidation_threshold,
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );
            assert_eq!(contract.underlying(), Some(underlying));
            assert_eq!(contract.controller(), Some(controller));
            assert_eq!(contract.manager(), Some(accounts.bob));
            assert_eq!(
                contract.incentives_controller(),
                Some(incentives_controller)
            );
            assert_eq!(
                contract.initial_exchange_rate_mantissa(),
                initial_exchange_rate_mantissa
            );
            assert_eq!(
                contract.reserve_factor_mantissa(),
                WrappedU256::from(U256::from(0))
            );
            assert_eq!(contract.total_borrows(), 0);
            assert_eq!(contract.liquidation_threshold(), liquidation_threshold);
        }

        #[ink::test]
        #[should_panic(
            expected = "not implemented: off-chain environment does not support contract invocation"
        )]
        fn transfer_works_overridden() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
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
            let liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            contract
                .transfer_from(accounts.bob, accounts.charlie, 0, Vec::new())
                .unwrap();
        }

        #[ink::test]
        fn set_controller_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
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
            let liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
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
            let liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
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
            let liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
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
            let liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
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

        #[ink::test]
        fn set_liquidation_threshold_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let dummy_id = AccountId::from([0x01; 32]);
            let mut liquidation_threshold = 10000;
            let mut contract = PoolContract::new(
                dummy_id,
                dummy_id,
                dummy_id,
                dummy_id,
                WrappedU256::from(U256::from(0)),
                liquidation_threshold,
                String::from("Token Name"),
                String::from("symbol"),
                8,
            );

            liquidation_threshold = 8000;
            let _ = contract.set_liquidation_threshold(liquidation_threshold);
            assert_eq!(contract.liquidation_threshold(), liquidation_threshold);
        }
    }
}
