// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

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
        pub minter: AccountId,
        pub mint_amount: Balance,
        pub mint_tokens: Balance,
    }
    /// Event: Execute `Redeem`
    #[ink(event)]
    pub struct Redeem {
        pub redeemer: AccountId,
        pub redeem_amount: Balance,
    }
    /// Event: Execute `Borrow`
    #[ink(event)]
    pub struct Borrow {
        pub borrower: AccountId,
        pub borrow_amount: Balance,
        pub account_borrows: Balance,
        pub total_borrows: Balance,
    }
    /// Event: Execute `RepayBorrow`
    #[ink(event)]
    pub struct RepayBorrow {
        pub payer: AccountId,
        pub borrower: AccountId,
        pub repay_amount: Balance,
        pub account_borrows: Balance,
        pub total_borrows: Balance,
    }
    /// Event: Execute `LiquidateBorrow`
    #[ink(event)]
    pub struct LiquidateBorrow {
        pub liquidator: AccountId,
        pub borrower: AccountId,
        pub repay_amount: Balance,
        pub token_collateral: AccountId,
        pub seize_tokens: Balance,
    }
    /// Event: Adding to Reserves
    #[ink(event)]
    pub struct ReservesAdded {
        pub benefactor: AccountId,
        pub add_amount: Balance,
        pub new_total_reserves: Balance,
    }

    /// Event: Transfer Pool Token
    ///
    /// NOTE: Use event emitter included in PSP22 Interface
    /// [PSP22 | Brushfam](https://learn.brushfam.io/docs/OpenBrush/smart-contracts/PSP22/)
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        pub from: Option<AccountId>,
        #[ink(topic)]
        pub to: Option<AccountId>,
        pub value: Balance,
    }
    /// Event: Allowance of a spender for an owner is set
    ///
    /// NOTE: Use event emitter included in PSP22 Interface
    /// [PSP22 | Brushfam](https://learn.brushfam.io/docs/OpenBrush/smart-contracts/PSP22/)
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub spender: AccountId,
        pub value: Balance,
    }

    /// Event: Delegation Allowance for Borrowing is changed
    #[ink(event)]
    pub struct DelegateApproval {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub delegatee: AccountId,
        pub amount: Balance,
    }

    /// Event: User has enabled Reserve as Collateral
    #[ink(event)]
    pub struct ReserveUsedAsCollateralEnabled {
        #[ink(topic)]
        pub user: AccountId,
    }

    /// Event: User has disabled Reserve as Collateral
    #[ink(event)]
    pub struct ReserveUsedAsCollateralDisabled {
        #[ink(topic)]
        pub user: AccountId,
    }

    /// Event: Pool Manager changed
    #[ink(event)]
    pub struct ManagerAddressUpdated {
        #[ink(topic)]
        pub old: AccountId,
        #[ink(topic)]
        pub new: AccountId,
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

        fn _emit_manager_updated_event(&self, old: AccountId, new: AccountId) {
            self.env().emit_event(ManagerAddressUpdated { old, new })
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
            incentives_controller: Option<AccountId>,
            underlying: AccountId,
            controller: AccountId,
            rate_model: AccountId,
            manager: AccountId,
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
            if rate_model.is_zero() {
                panic!("rate model is zero address");
            }
            if let Some(_incentives_controller) = incentives_controller {
                if _incentives_controller.is_zero() {
                    panic!("incentives controller is zero address");
                }
            }

            let mut instance = Self::default();
            instance._initialize(
                incentives_controller,
                underlying,
                controller,
                manager,
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
            incentives_controller: Option<AccountId>,
            underlying: AccountId,
            controller: AccountId,
            rate_model: AccountId,
            manager: AccountId,
            initial_exchange_rate_mantissa: WrappedU256,
            liquidation_threshold: u128,
        ) -> Self {
            if underlying.is_zero() {
                panic!("underlying is zero address");
            }
            if controller.is_zero() {
                panic!("controller is zero address");
            }
            if rate_model.is_zero() {
                panic!("rate model is zero address");
            }
            if let Some(_incentives_controller) = incentives_controller {
                if _incentives_controller.is_zero() {
                    panic!("incentives controller is zero address");
                }
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
                manager,
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
            incentives_controller: Option<AccountId>,
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
            self.pool.incentives_controller = incentives_controller;
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
}
