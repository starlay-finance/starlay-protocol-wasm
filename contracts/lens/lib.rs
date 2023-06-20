#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

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
        underlying_asset_address: AccountId,
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
        manager: AccountId,
        oracle: AccountId,
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
            PoolMetadata {
                pool,
                pool_decimals: PSP22MetadataRef::token_decimals(&pool),
                underlying_asset_address,
                underlying_decimals: PSP22MetadataRef::token_decimals(&underlying_asset_address),
                underlying_symbol: PSP22MetadataRef::token_symbol(&underlying_asset_address)
                    .unwrap_or_default(),
                is_listed: ControllerRef::is_listed(&controller, pool),
                total_cash: PoolRef::get_cash_prior(&pool),
                total_supply: PSP22Ref::total_supply(&pool),
                total_borrows: PoolRef::total_borrows(&pool),
                total_reserves: PoolRef::total_reserves(&pool),
                exchange_rate_current: PoolRef::exchange_rate_current(&pool).unwrap_or_default(),
                supply_rate_per_msec: PoolRef::supply_rate_per_msec(&pool),
                borrow_rate_per_msec: PoolRef::borrow_rate_per_msec(&pool),
                collateral_factor_mantissa: ControllerRef::collateral_factor_mantissa(
                    &controller,
                    pool,
                )
                .unwrap_or_default(),
                reserve_factor_mantissa: PoolRef::reserve_factor_mantissa(&pool),
                borrow_cap: ControllerRef::borrow_cap(&controller, pool),
                mint_guardian_paused: ControllerRef::mint_guardian_paused(&controller, pool)
                    .unwrap_or_default(),
                borrow_guardian_paused: ControllerRef::borrow_guardian_paused(&controller, pool)
                    .unwrap_or_default(),
            }
        }

        fn _pool_balances(&self, pool: AccountId, account: AccountId) -> PoolBalances {
            let underlying = PoolRef::underlying(&pool);
            PoolBalances {
                pool,
                balance_of: PSP22Ref::balance_of(&pool, account),
                borrow_balance_current: PoolRef::borrow_balance_current(&pool, account)
                    .unwrap_or_default(),
                token_balance: PSP22Ref::balance_of(&underlying, account),
                token_allowance: PSP22Ref::allowance(&underlying, account, pool),
            }
        }

        fn _pool_underlying_price(&self, pool: AccountId) -> PoolUnderlyingPrice {
            let controller = PoolRef::controller(&pool);
            let underlying = PoolRef::underlying(&pool);
            let oracle = ControllerRef::oracle(&controller);
            PoolUnderlyingPrice {
                pool,
                underlying_price: PriceOracleRef::get_price(&oracle, underlying).unwrap(),
            }
        }

        fn _underlying_balance(&self, pool: &AccountId, account: AccountId) -> Balance {
            let underlying = PoolRef::underlying(pool);
            PSP22Ref::balance_of(&underlying, account)
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

            let _contract = LensContract::new();
        }
    }
}
