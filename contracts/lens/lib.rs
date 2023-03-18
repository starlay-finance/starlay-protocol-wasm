#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

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
        traits::Storage,
    };
    use scale::{
        Decode,
        Encode,
    };

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolMetadata {
        pool: AccountId,
        pool_decimals: u8,
        underlying_asset_address: AccountId,
        underlying_decimals: u8,
        is_listed: bool,
        total_cash: Balance,
        total_supply: Balance,
        total_borrows: Balance,
        total_reserves: Balance,
        exchange_rate_current: WrappedU256,
        supply_rate_per_sec: WrappedU256,
        borrow_rate_per_sec: WrappedU256,
        collateral_factor_mantissa: u128,
        reserve_factor_mantissa: WrappedU256,
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
        pub fn pool_balances(&self, pool: AccountId, account: AccountId) -> PoolBalances {
            self._pool_balances(pool, account)
        }

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
            let controller = PoolRef::controller(&pool);
            let underlying_asset_address = PoolRef::underlying(&pool);
            PoolMetadata {
                pool,
                pool_decimals: PSP22MetadataRef::token_decimals(&pool),
                underlying_asset_address,
                underlying_decimals: PSP22MetadataRef::token_decimals(&underlying_asset_address),
                is_listed: ControllerRef::is_listed(&controller, pool),
                total_cash: PoolRef::get_cash_prior(&pool),
                total_supply: PSP22Ref::total_supply(&pool),
                total_borrows: PoolRef::total_borrows(&pool),
                total_reserves: PoolRef::total_reserves(&pool),
                exchange_rate_current: PoolRef::exchange_rate_current(&pool).unwrap_or_default(),
                supply_rate_per_sec: PoolRef::supply_rate_per_msec(&pool),
                borrow_rate_per_sec: PoolRef::borrow_rate_per_msec(&pool),
                collateral_factor_mantissa: 0,
                // TODO ControllerRef::collateral_factor(&controller, pool),
                reserve_factor_mantissa: PoolRef::reserve_factor_mantissa(&pool),
                borrow_cap: 0,
                // TODO ControllerRef::borrow_cap(&controller, pool),
            }
        }

        fn _pool_balances(&self, pool: AccountId, account: AccountId) -> PoolBalances {
            let underlying = PoolRef::underlying(&pool);
            PoolBalances {
                pool,
                balance_of: PSP22Ref::balance_of(&pool, account),
                balance_of_underlying: PoolRef::balance_of_underlying_current(&pool, account)
                    .unwrap_or_default(),
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
                underlying_price: PriceOracleRef::get_price(&oracle, underlying),
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
