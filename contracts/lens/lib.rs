#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use ink::prelude::vec::Vec;
    use openbrush::traits::Storage;
    use scale::{
        Decode,
        Encode,
    };

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolMetadata {
        pool: AccountId,
        pool_decimals: u128,
        underlying_asset_address: AccountId,
        underlying_decimals: u128,
        is_listed: bool,
        total_cash: Balance,
        total_supply: Balance,
        total_borrows: Balance,
        total_reserves: Balance,
        exchange_rate_current: u128,
        supply_rate_per_sec: u128,
        borrow_rate_per_sec: u128,
        collateral_factor_mantissa: u128,
        reserve_factor_mantissa: u128,
        borrow_cap: u128,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolBalances {
        pool: AccountId,
        balance_of: Balance,
        balance_of_underlying: Balance,
        borrow_balance_current: Balance,
        token_balance: Balance,
        token_allowance: Balance,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolUnderlyingPrice {
        pool: AccountId,
        underlying_price: u128,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AccountLimits {
        pools: Vec<AccountId>,
        liquidity: Balance,
        shortfall: Balance,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct LensContract {}

    impl LensContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn pool_metadata(&self, pool: AccountId) -> PoolMetadata {
            self._pool_metadata(pool)
        }

        #[ink(message)]
        pub fn pool_metadata_all(&self, pools: Vec<AccountId>) -> Vec<PoolMetadata> {
            pools
                .iter()
                .map(|pool| self._pool_metadata(*pool))
                .collect()
        }

        #[ink(message)]
        pub fn pool_balances(&self, pool: AccountId) -> PoolBalances {
            self._pool_balances(pool)
        }

        #[ink(message)]
        pub fn pool_balances_all(&self, pools: Vec<AccountId>) -> Vec<PoolBalances> {
            pools
                .iter()
                .map(|pool| self._pool_balances(*pool))
                .collect()
        }

        #[ink(message)]
        pub fn pool_underlying_price(&self, pool: AccountId) -> PoolUnderlyingPrice {
            self._pool_underlying_price(pool)
        }

        #[ink(message)]
        pub fn pool_underlying_price_all(&self, pools: Vec<AccountId>) -> Vec<PoolUnderlyingPrice> {
            pools
                .iter()
                .map(|pool| self._pool_underlying_price(*pool))
                .collect()
        }

        fn _pool_metadata(&self, pool: AccountId) -> PoolMetadata {
            // TODO
            PoolMetadata {
                pool,
                pool_decimals: 0,
                underlying_asset_address: [0u8; 32].into(),
                underlying_decimals: 0,
                is_listed: false,
                total_cash: 0,
                total_supply: 0,
                total_borrows: 0,
                total_reserves: 0,
                exchange_rate_current: 0,
                supply_rate_per_sec: 0,
                borrow_rate_per_sec: 0,
                collateral_factor_mantissa: 0,
                reserve_factor_mantissa: 0,
                borrow_cap: 0,
            }
        }

        fn _pool_balances(&self, pool: AccountId) -> PoolBalances {
            // TODO
            PoolBalances {
                pool,
                balance_of: 0,
                balance_of_underlying: 0,
                borrow_balance_current: 0,
                token_balance: 0,
                token_allowance: 0,
            }
        }

        fn _pool_underlying_price(&self, pool: AccountId) -> PoolUnderlyingPrice {
            // TODO
            PoolUnderlyingPrice {
                pool,
                underlying_price: 0,
            }
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
