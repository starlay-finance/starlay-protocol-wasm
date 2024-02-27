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

/// Definition of Lens Contract
///
/// This is a contract to make it easier to get protocol status and data for the frontend
#[openbrush::contract]
pub mod contract {
    use ink::prelude::vec::Vec;
    use logics::traits::{
        controller::ControllerRef,
        pool::PoolRef,
        price_oracle::PriceOracleRef,
        types::WrappedU256,
    };
    use openbrush::{
        contracts::traits::psp22::{
            extensions::metadata::PSP22MetadataRef,
            PSP22Ref,
        },
        traits::{
            Storage,
            String,
        },
    };
    use scale::{
        Decode,
        Encode,
    };

    /// Metadata in the Pool
    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolMetadata {
        pool: AccountId,
        pool_decimals: u8,
        underlying_asset_address: Option<AccountId>,
        underlying_decimals: u8,
        underlying_symbol: String,
        is_listed: bool,
        total_cash: Balance,
        total_supply: Balance,
        total_borrows: Balance,
        total_reserves: Balance,
        exchange_rate_current: WrappedU256,
        supply_rate_per_msec: WrappedU256,
        borrow_rate_per_msec: WrappedU256,
        collateral_factor_mantissa: WrappedU256,
        reserve_factor_mantissa: WrappedU256,
        borrow_cap: Option<u128>,
        mint_guardian_paused: bool,
        borrow_guardian_paused: bool,
    }

    /// Pool's Balance Information
    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolBalances {
        pool: AccountId,
        balance_of: Balance,
        borrow_balance_current: Balance,
        balance_of_underlying: Balance,
        token_balance: Balance,
        token_allowance: Balance,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolUnderlyingPrice {
        pool: AccountId,
        /// underlying_price of the pool
        underlying_price: u128,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AccountLimits {
        pools: Vec<AccountId>,
        liquidity: Balance,
        shortfall: Balance,
    }

    /// Protocol's Configuration
    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Configuration {
        manager: Option<AccountId>,
        oracle: Option<AccountId>,
        seize_guardian_paused: bool,
        transfer_guardian_paused: bool,
        liquidation_incentive_mantissa: WrappedU256,
        close_factor_mantissa: WrappedU256,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct LensContract {}

    impl LensContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Get account_id of all managed Pools
        #[ink(message)]
        pub fn pools(&self, controller: AccountId) -> Vec<AccountId> {
            self._pools(controller)
        }

        /// Get metadata for the specified pool
        #[ink(message)]
        pub fn pool_metadata(&self, pool: AccountId) -> PoolMetadata {
            self._pool_metadata(pool)
        }

        /// Get metadata for the specified pools
        #[ink(message)]
        pub fn pool_metadata_all(&self, pools: Vec<AccountId>) -> Vec<PoolMetadata> {
            pools
                .iter()
                .map(|pool| self._pool_metadata(*pool))
                .collect()
        }

        /// Get balance information for a specified user in a pool
        #[ink(message)]
        pub fn pool_balances(&self, pool: AccountId, account: AccountId) -> PoolBalances {
            self._pool_balances(pool, account)
        }

        /// Get balance informations for a specified user in pools
        #[ink(message)]
        pub fn pool_balances_all(
            &self,
            pools: Vec<AccountId>,
            account: AccountId,
        ) -> Vec<PoolBalances> {
            pools
                .iter()
                .map(|pool| self._pool_balances(*pool, account))
                .collect()
        }

        /// Get balance in underlying asset for a specified user in a pool
        #[ink(message)]
        pub fn underlying_balance(&self, pool: AccountId, account: AccountId) -> Balance {
            self._underlying_balance(&pool, account)
        }

        /// Get balances in underlying asset for a specified user in pools
        #[ink(message)]
        pub fn underlying_balance_all(
            &self,
            pools: Vec<AccountId>,
            account: AccountId,
        ) -> Vec<Balance> {
            pools
                .iter()
                .map(|pool| self._underlying_balance(pool, account))
                .collect()
        }

        /// Get underlying price for a specified pool
        #[ink(message)]
        pub fn pool_underlying_price(&self, pool: AccountId) -> PoolUnderlyingPrice {
            self._pool_underlying_price(pool)
        }

        /// Get underlying prices for specified pools
        #[ink(message)]
        pub fn pool_underlying_price_all(&self, pools: Vec<AccountId>) -> Vec<PoolUnderlyingPrice> {
            pools
                .iter()
                .map(|pool| self._pool_underlying_price(*pool))
                .collect()
        }

        /// Get protocol's configuration
        #[ink(message)]
        pub fn configuration(&self, controller: AccountId) -> Configuration {
            Configuration {
                manager: ControllerRef::manager(&controller),
                oracle: ControllerRef::oracle(&controller),
                seize_guardian_paused: ControllerRef::seize_guardian_paused(&controller),
                transfer_guardian_paused: ControllerRef::transfer_guardian_paused(&controller),
                liquidation_incentive_mantissa: ControllerRef::liquidation_incentive_mantissa(
                    &controller,
                ),
                close_factor_mantissa: ControllerRef::close_factor_mantissa(&controller),
            }
        }

        fn _pools(&self, controller: AccountId) -> Vec<AccountId> {
            ControllerRef::markets(&controller)
        }

        fn _pool_metadata(&self, pool: AccountId) -> PoolMetadata {
            let controller = PoolRef::controller(&pool);
            let underlying_asset_address = PoolRef::underlying(&pool);
            let (underlying_decimals, underlying_symbol) = if let Some(_underlying_asset_address) =
                underlying_asset_address
            {
                (
                    PSP22MetadataRef::token_decimals(&_underlying_asset_address),
                    PSP22MetadataRef::token_symbol(&_underlying_asset_address).unwrap_or_default(),
                )
            } else {
                (0_u8, String::from(""))
            };

            let (
                is_listed,
                collateral_factor_mantissa,
                borrow_cap,
                mint_guardian_paused,
                borrow_guardian_paused,
            ) = if let Some(_controller) = controller {
                (
                    ControllerRef::is_listed(&_controller, pool),
                    ControllerRef::collateral_factor_mantissa(&_controller, pool)
                        .unwrap_or_default(),
                    ControllerRef::borrow_cap(&_controller, pool),
                    ControllerRef::mint_guardian_paused(&_controller, pool).unwrap_or_default(),
                    ControllerRef::borrow_guardian_paused(&_controller, pool).unwrap_or_default(),
                )
            } else {
                (false, Default::default(), Some(0), true, true)
            };

            PoolMetadata {
                pool,
                pool_decimals: PSP22MetadataRef::token_decimals(&pool),
                underlying_asset_address,
                underlying_decimals,
                underlying_symbol,
                is_listed,
                total_cash: PoolRef::get_cash_prior(&pool),
                total_supply: PSP22Ref::total_supply(&pool),
                total_borrows: PoolRef::total_borrows(&pool),
                total_reserves: PoolRef::total_reserves(&pool),
                exchange_rate_current: PoolRef::exchange_rate_current(&pool).unwrap_or_default(),
                supply_rate_per_msec: PoolRef::supply_rate_per_msec(&pool),
                borrow_rate_per_msec: PoolRef::borrow_rate_per_msec(&pool),
                collateral_factor_mantissa,
                reserve_factor_mantissa: PoolRef::reserve_factor_mantissa(&pool),
                borrow_cap,
                mint_guardian_paused,
                borrow_guardian_paused,
            }
        }

        fn _pool_balances(&self, pool: AccountId, account: AccountId) -> PoolBalances {
            let underlying = PoolRef::underlying(&pool);
            let (token_balance, token_allowance) = if let Some(_underlying) = underlying {
                (
                    PSP22Ref::balance_of(&_underlying, account),
                    PSP22Ref::allowance(&_underlying, account, pool),
                )
            } else {
                (0, 0)
            };
            PoolBalances {
                pool,
                balance_of: PSP22Ref::balance_of(&pool, account),
                borrow_balance_current: PoolRef::borrow_balance_current(&pool, account)
                    .unwrap_or_default(),
                balance_of_underlying: PoolRef::balance_of_underlying(&pool, account),
                token_balance,
                token_allowance,
            }
        }

        fn _pool_underlying_price(&self, pool: AccountId) -> PoolUnderlyingPrice {
            let controller = PoolRef::controller(&pool);
            let underlying = PoolRef::underlying(&pool);

            if controller.is_none() || underlying.is_none() {
                return PoolUnderlyingPrice {
                    pool,
                    underlying_price: 0,
                }
            }

            let oracle = ControllerRef::oracle(&controller.unwrap());
            if oracle.is_none() {
                return PoolUnderlyingPrice {
                    pool,
                    underlying_price: 0,
                }
            }

            PoolUnderlyingPrice {
                pool,
                underlying_price: PriceOracleRef::get_price(&oracle.unwrap(), underlying.unwrap())
                    .unwrap(),
            }
        }

        fn _underlying_balance(&self, pool: &AccountId, account: AccountId) -> Balance {
            let underlying = PoolRef::underlying(pool);
            if let Some(_underlying) = underlying {
                return PSP22Ref::balance_of(&_underlying, account)
            }
            0
        }
    }
}
