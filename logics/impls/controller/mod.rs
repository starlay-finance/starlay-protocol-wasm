use super::{
    exp_no_err::Exp,
    pool::{
        COLLATERAL_FACTOR_MANTISSA_DECIMALS,
        LIQUIDATION_THRESHOLD_DECIMALS,
    },
};
pub use crate::traits::{
    controller::*,
    pool::{
        PoolMetaData,
        PoolRef,
    },
};
use crate::{
    impls::price_oracle::PRICE_PRECISION,
    traits::{
        price_oracle::PriceOracleRef,
        types::WrappedU256,
    },
};
use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};
use ink::prelude::vec::Vec;
use openbrush::{
    storage::Mapping,
    traits::{
        AccountId,
        Balance,
        Storage,
        String,
    },
};
use primitive_types::U256;
mod utils;
pub use self::utils::{
    balance_decrease_allowed,
    calculate_available_borrow_in_base_currency,
    calculate_health_factor_from_balances,
    collateral_factor_max_mantissa,
    get_hypothetical_account_liquidity,
    liquidate_calculate_seize_tokens,
    BalanceDecreaseAllowedParam,
    GetHypotheticalAccountLiquidityInput,
    HypotheticalAccountLiquidityCalculationParam,
    LiquidateCalculateSeizeTokensInput,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

pub const MAXIMUM_MARKETS: usize = 8;

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// AccountId of managed Pools
    pub markets: Vec<AccountId>,
    /// Pair of underlying and pool
    pub underlying_market_pair: Mapping<AccountId, AccountId>,
    /// Pair of pool and underlying
    pub market_underlying_pair: Mapping<AccountId, AccountId>,
    /// Mapping of Pool and Collateral Factors (Decimals: 18)
    pub collateral_factor_mantissa: Mapping<AccountId, WrappedU256>,
    /// Whether Pool has paused `Mint` Action
    pub mint_guardian_paused: Mapping<AccountId, bool>,
    /// Whether Pool has paused `Borrow` Action
    pub borrow_guardian_paused: Mapping<AccountId, bool>,
    /// Whether Pool has paused `Seize` Action
    pub seize_guardian_paused: bool,
    /// Whether Pool has paused `Transfer` Action
    pub transfer_guardian_paused: bool,
    /// Oracle's AccountId associated with this contract
    pub oracle: Option<AccountId>,
    /// Close Factor
    pub close_factor_mantissa: WrappedU256,
    /// Liquidation Incentive
    pub liquidation_incentive_mantissa: WrappedU256,
    /// Maximum that can be borrowed per Pool
    pub borrow_caps: Mapping<AccountId, Balance>,
    /// Manager's AccountId associated with this contract
    pub manager: Option<AccountId>,
    /// AccountId of Pending Manager use for transfer manager role
    pub pending_manager: Option<AccountId>,
    /// Flashloan Gateway's AccountId associated with this contract
    pub flashloan_gateway: Option<AccountId>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            markets: Default::default(),
            underlying_market_pair: Default::default(),
            market_underlying_pair: Default::default(),
            collateral_factor_mantissa: Default::default(),
            mint_guardian_paused: Default::default(),
            borrow_guardian_paused: Default::default(),
            seize_guardian_paused: Default::default(),
            transfer_guardian_paused: Default::default(),
            oracle: None,
            close_factor_mantissa: WrappedU256::from(U256::zero()),
            liquidation_incentive_mantissa: WrappedU256::from(U256::zero()),
            borrow_caps: Default::default(),
            manager: None,
            pending_manager: None,
            flashloan_gateway: None,
        }
    }
}

pub trait Internal {
    fn _mint_allowed(&self, pool: AccountId, minter: AccountId, mint_amount: Balance)
        -> Result<()>;

    fn _redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()>;

    fn _borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()>;

    fn _liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()>;

    fn _seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;

    fn _transfer_allowed(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()>;

    fn _liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        exchange_rate_mantissa: WrappedU256,
        repay_amount: Balance,
        pool_borrowed_attributes: Option<PoolAttributesForSeizeCalculation>,
        pool_collateral_attributes: Option<PoolAttributesForSeizeCalculation>,
    ) -> Result<Balance>;
    fn _assert_manager(&self) -> Result<()>;
    fn _assert_pending_manager(&self) -> Result<()>;

    // admin functions
    fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;
    fn _support_market(
        &mut self,
        pool: &AccountId,
        underlying: &AccountId,
        collateral_factor_mantissa: Option<WrappedU256>,
    ) -> Result<()>;
    fn _set_flashloan_gateway(&mut self, flashloan_gateway: AccountId) -> Result<()>;
    fn _set_collateral_factor_mantissa(
        &mut self,
        pool: &AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()>;
    fn _set_mint_guardian_paused(&mut self, pool: &AccountId, paused: bool) -> Result<()>;
    fn _set_borrow_guardian_paused(&mut self, pool: &AccountId, paused: bool) -> Result<()>;
    fn _set_seize_guardian_paused(&mut self, paused: bool) -> Result<()>;
    fn _set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()>;
    fn _set_close_factor_mantissa(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()>;
    fn _set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()>;
    fn _set_borrow_cap(&mut self, pool: &AccountId, new_cap: Balance) -> Result<()>;
    fn _set_manager(&mut self, manager: AccountId) -> Result<()>;
    fn _accept_manager(&mut self) -> Result<()>;

    // view function
    fn _markets(&self) -> Vec<AccountId>;
    fn _market_of_underlying(&self, underlying: AccountId) -> Option<AccountId>;
    fn _underlying_of_market(&self, pool: AccountId) -> Option<AccountId>;
    fn _flashloan_gateway(&self) -> Option<AccountId>;
    fn _collateral_factor_mantissa(&self, pool: AccountId) -> Option<WrappedU256>;
    fn _is_listed(&self, pool: AccountId) -> bool;
    fn _mint_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    fn _borrow_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    fn _seize_guardian_paused(&self) -> bool;
    fn _transfer_guardian_paused(&self) -> bool;
    fn _oracle(&self) -> Option<AccountId>;
    fn _close_factor_mantissa(&self) -> WrappedU256;
    fn _liquidation_incentive_mantissa(&self) -> WrappedU256;
    fn _borrow_cap(&self, pool: AccountId) -> Option<Balance>;
    fn _manager(&self) -> Option<AccountId>;
    fn _pending_manager(&self) -> Option<AccountId>;
    fn _account_assets(
        &self,
        account: AccountId,
        token_modify: Option<AccountId>,
    ) -> Vec<AccountId>;
    fn _get_account_liquidity(&self, account: AccountId) -> Result<(U256, U256)>;
    fn _get_hypothetical_account_liquidity(
        &self,
        account: AccountId,
        token: Option<AccountId>,
        redeem_tokens: Balance,
        borrow_amount: Balance,
        pool_attributes: Option<PoolAttributes>,
    ) -> Result<(U256, U256)>;
    fn _calculate_user_account_data(
        &self,
        account: AccountId,
        pool_attributes: Option<PoolAttributes>,
        token_modify: Option<AccountId>,
    ) -> Result<(
        AccountCollateralData,
        Vec<HypotheticalAccountLiquidityCalculationParam>,
    )>;
    fn _balance_decrease_allowed(
        &self,
        pool_attributes: PoolAttributes,
        account: AccountId,
        amount: Balance,
    ) -> Result<()>;

    // event emission
    fn _emit_market_listed_event(&self, pool: AccountId);
    fn _emit_new_collateral_factor_event(
        &self,
        pool: AccountId,
        old: WrappedU256,
        new: WrappedU256,
    );
    fn _emit_pool_action_paused_event(&self, pool: AccountId, action: String, paused: bool);
    fn _emit_action_paused_event(&self, action: String, paused: bool);
    fn _emit_new_price_oracle_event(&self, old: Option<AccountId>, new: Option<AccountId>);
    fn _emit_new_flashloan_gateway_event(&self, _old: Option<AccountId>, _new: Option<AccountId>);
    fn _emit_new_close_factor_event(&self, old: WrappedU256, new: WrappedU256);
    fn _emit_new_liquidation_incentive_event(&self, old: WrappedU256, new: WrappedU256);
    fn _emit_new_borrow_cap_event(&self, pool: AccountId, new: Balance);
    fn _emit_manager_updated_event(&self, old: AccountId, new: AccountId);
}

impl<T: Storage<Data>> Controller for T {
    default fn mint_allowed(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
    ) -> Result<()> {
        self._mint_allowed(pool, minter, mint_amount)
    }

    default fn redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()> {
        self._redeem_allowed(pool, redeemer, redeem_amount, pool_attribute)
    }

    default fn borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()> {
        self._borrow_allowed(pool, borrower, borrow_amount, pool_attribute)
    }

    default fn liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()> {
        self._liquidate_borrow_allowed(
            pool_borrowed,
            pool_collateral,
            liquidator,
            borrower,
            repay_amount,
            pool_attribute,
        )
    }

    default fn seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()> {
        self._seize_allowed(
            pool_collateral,
            pool_borrowed,
            liquidator,
            borrower,
            seize_tokens,
        )
    }

    default fn transfer_allowed(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()> {
        self._transfer_allowed(pool, src, dst, transfer_tokens, pool_attribute)
    }

    default fn liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        exchange_rate_mantissa: WrappedU256,
        repay_amount: Balance,
        pool_borrowed_attributes: Option<PoolAttributesForSeizeCalculation>,
        pool_collateral_attributes: Option<PoolAttributesForSeizeCalculation>,
    ) -> Result<Balance> {
        self._liquidate_calculate_seize_tokens(
            pool_borrowed,
            pool_collateral,
            exchange_rate_mantissa,
            repay_amount,
            pool_borrowed_attributes,
            pool_collateral_attributes,
        )
    }

    default fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        self._assert_manager()?;
        let old = self._oracle();
        self._set_price_oracle(new_oracle)?;
        self._emit_new_price_oracle_event(old, Some(new_oracle));
        Ok(())
    }

    default fn support_market(&mut self, pool: AccountId, underlying: AccountId) -> Result<()> {
        self._assert_manager()?;
        self._support_market(&pool, &underlying, None)?;
        self._emit_market_listed_event(pool);
        Ok(())
    }

    default fn set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()> {
        self._assert_manager()?;
        let old = self._flashloan_gateway();
        self._set_flashloan_gateway(new_flashloan_gateway)?;
        self._emit_new_flashloan_gateway_event(old, Some(new_flashloan_gateway));
        Ok(())
    }

    default fn support_market_with_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        underlying: AccountId,
        collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        self._support_market(&pool, &underlying, Some(collateral_factor_mantissa))?;
        self._emit_market_listed_event(pool);
        Ok(())
    }

    default fn set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        let old = self._collateral_factor_mantissa(pool).unwrap_or_default();
        self._set_collateral_factor_mantissa(&pool, new_collateral_factor_mantissa)?;
        self._emit_new_collateral_factor_event(pool, old, new_collateral_factor_mantissa);
        Ok(())
    }

    default fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_mint_guardian_paused(&pool, paused)?;
        self._emit_pool_action_paused_event(pool, String::from("Mint"), paused);
        Ok(())
    }

    default fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_borrow_guardian_paused(&pool, paused)?;
        self._emit_pool_action_paused_event(pool, String::from("Borrow"), paused);
        Ok(())
    }

    default fn set_seize_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_seize_guardian_paused(paused)?;
        self._emit_action_paused_event(String::from("Seize"), paused);
        Ok(())
    }

    default fn set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_transfer_guardian_paused(paused)?;
        self._emit_action_paused_event(String::from("Transfer"), paused);
        Ok(())
    }

    default fn set_close_factor_mantissa(
        &mut self,
        new_close_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        let old = self._close_factor_mantissa();
        self._set_close_factor_mantissa(new_close_factor_mantissa)?;
        self._emit_new_close_factor_event(old, new_close_factor_mantissa);
        Ok(())
    }

    default fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        let old = self._liquidation_incentive_mantissa();
        self._set_liquidation_incentive_mantissa(new_liquidation_incentive_mantissa)?;
        self._emit_new_liquidation_incentive_event(old, new_liquidation_incentive_mantissa);
        Ok(())
    }

    default fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()> {
        self._assert_manager()?;
        self._set_borrow_cap(&pool, new_cap)?;
        self._emit_new_borrow_cap_event(pool, new_cap);
        Ok(())
    }

    default fn set_manager(&mut self, manager: AccountId) -> Result<()> {
        self._assert_manager()?;
        self._set_manager(manager)?;
        Ok(())
    }

    default fn accept_manager(&mut self) -> Result<()> {
        self._assert_pending_manager()?;
        self._accept_manager()?;
        Ok(())
    }

    default fn markets(&self) -> Vec<AccountId> {
        self._markets()
    }

    default fn market_of_underlying(&self, underlying: AccountId) -> Option<AccountId> {
        self._market_of_underlying(underlying)
    }

    default fn underlying_of_market(&self, pool: AccountId) -> Option<AccountId> {
        self._underlying_of_market(pool)
    }

    default fn flashloan_gateway(&self) -> Option<AccountId> {
        self._flashloan_gateway()
    }

    default fn collateral_factor_mantissa(&self, pool: AccountId) -> Option<WrappedU256> {
        self._collateral_factor_mantissa(pool)
    }

    default fn mint_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self._mint_guardian_paused(pool)
    }

    default fn borrow_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self._borrow_guardian_paused(pool)
    }

    default fn seize_guardian_paused(&self) -> bool {
        self._seize_guardian_paused()
    }

    default fn transfer_guardian_paused(&self) -> bool {
        self._transfer_guardian_paused()
    }

    default fn oracle(&self) -> Option<AccountId> {
        self._oracle()
    }

    default fn close_factor_mantissa(&self) -> WrappedU256 {
        self._close_factor_mantissa()
    }

    default fn liquidation_incentive_mantissa(&self) -> WrappedU256 {
        self._liquidation_incentive_mantissa()
    }

    default fn borrow_cap(&self, pool: AccountId) -> Option<Balance> {
        self._borrow_cap(pool)
    }

    default fn manager(&self) -> Option<AccountId> {
        self._manager()
    }

    default fn pending_manager(&self) -> Option<AccountId> {
        self._pending_manager()
    }

    default fn is_listed(&self, pool: AccountId) -> bool {
        self._is_listed(pool)
    }

    default fn account_assets(&self, account: AccountId) -> Vec<AccountId> {
        self._account_assets(account, None)
    }

    default fn get_account_liquidity(&self, account: AccountId) -> Result<(U256, U256)> {
        self._get_account_liquidity(account)
    }

    default fn get_hypothetical_account_liquidity(
        &self,
        account: AccountId,
        token: AccountId,
        redeem_tokens: Balance,
        borrow_amount: Balance,
    ) -> Result<(U256, U256)> {
        self._get_hypothetical_account_liquidity(
            account,
            Some(token),
            redeem_tokens,
            borrow_amount,
            None,
        )
    }

    default fn calculate_user_account_data(
        &self,
        account: AccountId,
        pool_attributes: Option<PoolAttributes>,
    ) -> Result<AccountData> {
        let (account_data, _) =
            self._calculate_user_account_data(account, pool_attributes, None)?;

        Ok(AccountData {
            total_collateral_in_base_currency: account_data.total_collateral_in_base_currency,
            total_debt_in_base_currency: account_data.total_debt_in_base_currency,
            avg_ltv: account_data.avg_ltv,
            avg_liquidation_threshold: account_data.avg_liquidation_threshold,
            health_factor: account_data.health_factor,
        })
    }

    default fn balance_decrease_allowed(
        &self,
        pool_attributes: PoolAttributes,
        account: AccountId,
        amount: Balance,
    ) -> Result<()> {
        self._balance_decrease_allowed(pool_attributes, account, amount)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _mint_allowed(
        &self,
        pool: AccountId,
        _minter: AccountId,
        _mint_amount: Balance,
    ) -> Result<()> {
        if let Some(true) | None = self._mint_guardian_paused(pool) {
            return Err(Error::MintIsPaused)
        }
        // FEATURE: update governance token supply index & distribute

        Ok(())
    }

    default fn _redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        pool_attributes: Option<PoolAttributes>,
    ) -> Result<()> {
        if !self._is_listed(pool) {
            return Err(Error::MarketNotListed)
        }

        let (
            AccountCollateralData {
                total_collateral_in_base_currency,
                avg_ltv: _,
                avg_liquidation_threshold,
                total_debt_in_base_currency,
                asset_price,
                liquidation_threshold,
                health_factor: _,
            },
            asset_params,
        ) = self._calculate_user_account_data(redeemer, pool_attributes, Some(pool))?;

        // Prepare parameters for calculation
        let (sum_collateral, sum_borrow_plus_effect) =
            get_hypothetical_account_liquidity(GetHypotheticalAccountLiquidityInput {
                asset_params,
                token_modify: Some(pool),
                redeem_tokens: redeem_amount,
                borrow_amount: 0,
            });

        // These are safe, as the underflow condition is checked first
        if sum_collateral < sum_borrow_plus_effect {
            return Err(Error::InsufficientLiquidity)
        }

        if total_debt_in_base_currency.is_zero() {
            return Ok(())
        }

        let balance_decrease_allowed_result =
            balance_decrease_allowed(BalanceDecreaseAllowedParam {
                total_collateral_in_base_currency,
                total_debt_in_base_currency,
                avg_liquidation_threshold,
                amount_in_base_currency_unit: U256::from(redeem_amount),
                asset_price: U256::from(asset_price),
                liquidation_threshold: U256::from(liquidation_threshold),
            });

        if !balance_decrease_allowed_result {
            return Err(Error::BalanceDecreaseNotAllowed)
        }

        // FEATURE: update governance token supply index & distribute
        Ok(())
    }

    default fn _borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()> {
        if !self._is_listed(pool) {
            return Err(Error::MarketNotListed)
        }

        if let Some(true) | None = self._borrow_guardian_paused(pool) {
            return Err(Error::BorrowIsPaused)
        }

        let oracle = self._oracle().ok_or(Error::OracleIsNotSet)?;
        let (price, total_borrow, pool_attributes) = if let Some(attrs) = pool_attribute {
            let underlying = attrs.underlying.ok_or(Error::UnderlyingIsNotSet)?;
            (
                PriceOracleRef::get_price(&oracle, underlying),
                attrs.total_borrows,
                Some(attrs),
            )
        } else {
            (
                PriceOracleRef::get_underlying_price(&oracle, pool),
                PoolRef::total_borrows(&pool),
                None,
            )
        };

        if let None | Some(0) = price {
            return Err(Error::PriceError)
        }
        let borrow_cap = self._borrow_cap(pool).unwrap_or_default();
        if borrow_cap != 0 {
            if borrow_cap < borrow_amount || total_borrow > borrow_cap - borrow_amount {
                return Err(Error::BorrowCapReached)
            }
        }

        let (_, shortfall) = self._get_hypothetical_account_liquidity(
            borrower,
            Some(pool),
            0,
            borrow_amount,
            pool_attributes,
        )?;
        if !shortfall.is_zero() {
            return Err(Error::InsufficientLiquidity)
        }

        // FEATURE: update governance token borrow index & distribute

        Ok(())
    }

    default fn _liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        _liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()> {
        if !self._is_listed(pool_borrowed) || !self._is_listed(pool_collateral) {
            return Err(Error::MarketNotListed)
        }

        let (borrow_balance, pool_attributes) = if let Some(attrs) = pool_attribute.clone() {
            (attrs.account_borrow_balance, Some(attrs))
        } else {
            (
                PoolRef::borrow_balance_stored(&pool_borrowed, borrower),
                None,
            )
        };

        // The borrower must have shortfall in order to be liquidatable
        let (_, shortfall) =
            self._get_hypothetical_account_liquidity(borrower, None, 0, 0, pool_attributes)?;
        if shortfall.is_zero() {
            return Err(Error::InsufficientShortfall)
        }

        // The liquidator may not repay more than what is allowed by the closeFactor
        let max_close = Exp {
            mantissa: self._close_factor_mantissa(),
        }
        .mul_scalar_truncate(U256::from(borrow_balance));
        if U256::from(repay_amount).gt(&max_close) {
            return Err(Error::TooMuchRepay)
        }

        Ok(())
    }

    default fn _seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        _liquidator: AccountId,
        _borrower: AccountId,
        _seize_tokens: Balance,
    ) -> Result<()> {
        if self._seize_guardian_paused() {
            return Err(Error::SeizeIsPaused)
        }

        if !self._is_listed(pool_collateral) || !self._is_listed(pool_borrowed) {
            return Err(Error::MarketNotListed)
        }

        // NOTE: cannot perform controller check on the pool here, as a cross-contract call to the caller occurs when the pool is the caller.
        //   To avoid this, the pool itself needs to perform this check.
        // let p_collateral_ctrler = PoolRef::controller(&pool_collateral);
        // let p_borrowed_ctrler = PoolRef::controller(&pool_borrowed);
        // if p_collateral_ctrler != p_borrowed_ctrler {
        //     return Err(Error::ControllerMismatch)
        // }

        // FEATURE: update governance token supply index & distribute to borrower,liquidator

        Ok(())
    }

    default fn _transfer_allowed(
        &self,
        pool: AccountId,
        src: AccountId,
        _dst: AccountId,
        transfer_tokens: Balance,
        pool_attribute: Option<PoolAttributes>,
    ) -> Result<()> {
        if self._transfer_guardian_paused() {
            return Err(Error::TransferIsPaused)
        }

        self._redeem_allowed(pool, src, transfer_tokens, pool_attribute)?;

        Ok(())
    }

    default fn _liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        exchange_rate_mantissa: WrappedU256,
        repay_amount: Balance,
        pool_borrowed_attributes: Option<PoolAttributesForSeizeCalculation>,
        pool_collateral_attributes: Option<PoolAttributesForSeizeCalculation>,
    ) -> Result<Balance> {
        let oracle = self._oracle().ok_or(Error::OracleIsNotSet)?;
        let (price_borrowed_mantissa, pool_decimals_borrowed) =
            if let Some(attrs) = pool_borrowed_attributes {
                let underlying = attrs.underlying.ok_or(Error::UnderlyingIsNotSet)?;
                (
                    PriceOracleRef::get_price(&oracle, underlying).ok_or(Error::PriceError)?,
                    attrs.decimals,
                )
            } else {
                (
                    PriceOracleRef::get_underlying_price(&oracle, pool_borrowed)
                        .ok_or(Error::PriceError)?,
                    PoolRef::token_decimals(&pool_borrowed),
                )
            };
        if price_borrowed_mantissa == 0 {
            return Err(Error::PriceError)
        }

        let (price_collateral_mantissa, pool_decimals_collateral) =
            if let Some(attrs) = pool_collateral_attributes {
                let underlying = attrs.underlying.ok_or(Error::UnderlyingIsNotSet)?;
                (
                    PriceOracleRef::get_price(&oracle, underlying).ok_or(Error::PriceError)?,
                    attrs.decimals,
                )
            } else {
                (
                    PriceOracleRef::get_underlying_price(&oracle, pool_collateral)
                        .ok_or(Error::PriceError)?,
                    PoolRef::token_decimals(&pool_collateral),
                )
            };
        if price_collateral_mantissa == 0 {
            return Err(Error::PriceError)
        }

        let result = liquidate_calculate_seize_tokens(&LiquidateCalculateSeizeTokensInput {
            price_borrowed_mantissa: U256::from(price_borrowed_mantissa),
            decimals_borrowed: pool_decimals_borrowed,
            price_collateral_mantissa: U256::from(price_collateral_mantissa),
            decimals_collateral: pool_decimals_collateral,
            exchange_rate_mantissa: exchange_rate_mantissa.into(),
            liquidation_incentive_mantissa: self._liquidation_incentive_mantissa().into(),
            actual_repay_amount: repay_amount,
        });

        Ok(result)
    }

    default fn _assert_manager(&self) -> Result<()> {
        let manager = self._manager().ok_or(Error::ManagerIsNotSet)?;
        if Self::env().caller() != manager {
            return Err(Error::CallerIsNotManager)
        }

        Ok(())
    }

    default fn _assert_pending_manager(&self) -> Result<()> {
        let pending_manager = self
            ._pending_manager()
            .ok_or(Error::PendingManagerIsNotSet)?;
        if Self::env().caller() != pending_manager {
            return Err(Error::CallerIsNotPendingManager)
        }

        Ok(())
    }

    default fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        self.data().oracle = Some(new_oracle);
        Ok(())
    }

    default fn _set_flashloan_gateway(&mut self, new_flashloan_gateway: AccountId) -> Result<()> {
        self.data().flashloan_gateway = Some(new_flashloan_gateway);
        Ok(())
    }

    default fn _support_market(
        &mut self,
        pool: &AccountId,
        underlying: &AccountId,
        collateral_factor_mantissa: Option<WrappedU256>,
    ) -> Result<()> {
        // Prevent clone to reduce gas
        if self.data().markets.len() >= MAXIMUM_MARKETS {
            return Err(Error::MarketCountReachedToMaximum)
        }

        if self._is_listed(*pool) {
            return Err(Error::MarketAlreadyListed)
        }

        if let Some(_existing) = self.data().underlying_market_pair.get(underlying) {
            return Err(Error::MarketAlreadyListed)
        }

        self.data().markets.push(*pool);
        self.data().underlying_market_pair.insert(underlying, pool);
        self.data().market_underlying_pair.insert(pool, underlying);

        // set default states
        self._set_mint_guardian_paused(pool, false)?;
        self._set_borrow_guardian_paused(pool, false)?;
        if let Some(value) = collateral_factor_mantissa {
            self._set_collateral_factor_mantissa(pool, value)?;
        }
        self._set_borrow_cap(pool, 0)?;

        Ok(())
    }

    default fn _set_collateral_factor_mantissa(
        &mut self,
        pool: &AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        let new_collateral_factor_mantissa_u256 = U256::from(new_collateral_factor_mantissa);
        if new_collateral_factor_mantissa_u256.is_zero()
            || new_collateral_factor_mantissa_u256.gt(&collateral_factor_max_mantissa())
        {
            return Err(Error::InvalidCollateralFactor)
        }

        let liquidation_threshold: u128 = PoolRef::liquidation_threshold(pool);
        let liquidation_threshold_u256 = U256::from(liquidation_threshold).mul(U256::from(
            10_u128.pow(COLLATERAL_FACTOR_MANTISSA_DECIMALS - LIQUIDATION_THRESHOLD_DECIMALS),
        ));

        if new_collateral_factor_mantissa_u256.gt(&liquidation_threshold_u256) {
            return Err(Error::InvalidCollateralFactor)
        }

        let oracle = self._oracle().ok_or(Error::OracleIsNotSet)?;
        if let None | Some(0) = PriceOracleRef::get_underlying_price(&oracle, *pool) {
            return Err(Error::PriceError)
        }

        self.data()
            .collateral_factor_mantissa
            .insert(pool, &new_collateral_factor_mantissa);

        Ok(())
    }

    default fn _set_mint_guardian_paused(&mut self, pool: &AccountId, paused: bool) -> Result<()> {
        self.data().mint_guardian_paused.insert(pool, &paused);
        Ok(())
    }

    default fn _set_borrow_guardian_paused(
        &mut self,
        pool: &AccountId,
        paused: bool,
    ) -> Result<()> {
        self.data().borrow_guardian_paused.insert(pool, &paused);
        Ok(())
    }

    default fn _set_seize_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self.data().seize_guardian_paused = paused;
        Ok(())
    }

    default fn _set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self.data().transfer_guardian_paused = paused;
        Ok(())
    }

    default fn _set_close_factor_mantissa(
        &mut self,
        new_close_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self.data().close_factor_mantissa = new_close_factor_mantissa;
        Ok(())
    }

    default fn _set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        self.data().liquidation_incentive_mantissa = new_liquidation_incentive_mantissa;
        Ok(())
    }

    default fn _set_borrow_cap(&mut self, pool: &AccountId, new_cap: Balance) -> Result<()> {
        self.data().borrow_caps.insert(pool, &new_cap);
        Ok(())
    }

    default fn _set_manager(&mut self, manager: AccountId) -> Result<()> {
        self.data().pending_manager = Some(manager);
        Ok(())
    }

    default fn _accept_manager(&mut self) -> Result<()> {
        let manager = self._manager().ok_or(Error::ManagerIsNotSet)?;
        let pending_manager = self
            ._pending_manager()
            .ok_or(Error::PendingManagerIsNotSet)?;
        self.data().manager = Some(pending_manager);
        self.data().pending_manager = None;

        self._emit_manager_updated_event(manager, pending_manager);
        Ok(())
    }

    default fn _markets(&self) -> Vec<AccountId> {
        self.data().markets.clone()
    }

    default fn _market_of_underlying(&self, underlying: AccountId) -> Option<AccountId> {
        self.data().underlying_market_pair.get(&underlying)
    }

    default fn _underlying_of_market(&self, pool: AccountId) -> Option<AccountId> {
        self.data().market_underlying_pair.get(&pool)
    }

    default fn _flashloan_gateway(&self) -> Option<AccountId> {
        self.data().flashloan_gateway
    }

    default fn _is_listed(&self, pool: AccountId) -> bool {
        if let Some(_underlying) = self.data().market_underlying_pair.get(&pool) {
            return true
        }

        return false
    }

    default fn _collateral_factor_mantissa(&self, pool: AccountId) -> Option<WrappedU256> {
        self.data().collateral_factor_mantissa.get(&pool)
    }

    default fn _mint_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self.data().mint_guardian_paused.get(&pool)
    }

    default fn _borrow_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self.data().borrow_guardian_paused.get(&pool)
    }

    default fn _seize_guardian_paused(&self) -> bool {
        self.data().seize_guardian_paused
    }

    default fn _transfer_guardian_paused(&self) -> bool {
        self.data().transfer_guardian_paused
    }

    default fn _oracle(&self) -> Option<AccountId> {
        self.data().oracle
    }

    default fn _close_factor_mantissa(&self) -> WrappedU256 {
        self.data::<Data>().close_factor_mantissa
    }

    default fn _liquidation_incentive_mantissa(&self) -> WrappedU256 {
        self.data::<Data>().liquidation_incentive_mantissa
    }

    default fn _borrow_cap(&self, pool: AccountId) -> Option<Balance> {
        self.data().borrow_caps.get(&pool)
    }

    default fn _manager(&self) -> Option<AccountId> {
        self.data().manager
    }

    default fn _pending_manager(&self) -> Option<AccountId> {
        self.data().pending_manager
    }

    default fn _account_assets(
        &self,
        account: AccountId,
        token_modify: Option<AccountId>,
    ) -> Vec<AccountId> {
        let mut account_assets = Vec::<AccountId>::new();
        for pool in self._markets() {
            if pool == Self::env().caller() {
                continue // NOTE: if caller is pool, need to check by the pool itself
            }
            if token_modify.is_some() && pool == token_modify.unwrap() {
                account_assets.push(pool); // NOTE: add unconditionally even if balance, borrowed is not already there
                continue
            }
            let (balance, borrowed, _) = PoolRef::get_account_snapshot(&pool, account);

            // whether deposits or loans exist
            if balance > 0 || borrowed > 0 {
                account_assets.push(pool);
            }
        }
        return account_assets
    }

    default fn _get_account_liquidity(&self, account: AccountId) -> Result<(U256, U256)> {
        self._get_hypothetical_account_liquidity(account, None, 0, 0, None)
    }

    default fn _get_hypothetical_account_liquidity(
        &self,
        account: AccountId,
        token_modify: Option<AccountId>,
        redeem_tokens: Balance,
        borrow_amount: Balance,
        pool_attributes: Option<PoolAttributes>,
    ) -> Result<(U256, U256)> {
        let (_, asset_params) =
            self._calculate_user_account_data(account, pool_attributes, token_modify)?;

        let (sum_collateral, sum_borrow_plus_effect) =
            get_hypothetical_account_liquidity(GetHypotheticalAccountLiquidityInput {
                asset_params,
                token_modify,
                redeem_tokens,
                borrow_amount,
            });

        // These are safe, as the underflow condition is checked first
        let value = if sum_collateral > sum_borrow_plus_effect {
            (sum_collateral.sub(sum_borrow_plus_effect), U256::from(0))
        } else {
            (U256::from(0), sum_borrow_plus_effect.sub(sum_collateral))
        };

        Ok(value)
    }

    default fn _calculate_user_account_data(
        &self,
        account: AccountId,
        pool_attributes: Option<PoolAttributes>,
        token_modify: Option<AccountId>,
    ) -> Result<(
        AccountCollateralData,
        Vec<HypotheticalAccountLiquidityCalculationParam>,
    )> {
        let oracle = self._oracle().ok_or(Error::OracleIsNotSet)?;
        let caller = Self::env().caller();

        let mut total_collateral_in_base_currency = U256::from(0);
        let mut avg_ltv = U256::from(0);
        let mut avg_liquidation_threshold = U256::from(0);
        let mut total_debt_in_base_currency: U256 = U256::from(0);

        let mut asset_params = Vec::<HypotheticalAccountLiquidityCalculationParam>::new();

        let (asset_price, liquidation_threshold, skip_pool) = if let Some(pool_attribute) =
            pool_attributes
        {
            // if caller is a pool, get parameters for the pool without call the pool
            let attr_underlying = pool_attribute.underlying.ok_or(Error::UnderlyingIsNotSet)?;
            let attr_pool = pool_attribute.pool.ok_or(Error::PoolIsNotSet)?;

            let collateral_factor_mantissa = self
                ._collateral_factor_mantissa(attr_pool)
                .ok_or(Error::InvalidCollateralFactor)?;
            let ltv = U256::from(collateral_factor_mantissa);

            let oracle_price: u128 =
                PriceOracleRef::get_price(&oracle, attr_underlying).ok_or(Error::PriceError)?;
            if oracle_price == 0 {
                return Err(Error::PriceError)
            }
            let oracle_price_mantissa = Exp {
                mantissa: WrappedU256::from(U256::from(oracle_price)),
            };

            asset_params.push(HypotheticalAccountLiquidityCalculationParam {
                asset: attr_pool,
                decimals: pool_attribute.decimals,
                token_balance: pool_attribute.account_balance,
                borrow_balance: pool_attribute.account_borrow_balance,
                exchange_rate_mantissa: Exp {
                    mantissa: WrappedU256::from(pool_attribute.exchange_rate),
                },
                collateral_factor_mantissa: Exp {
                    mantissa: collateral_factor_mantissa,
                },
                oracle_price_mantissa: oracle_price_mantissa.clone(),
            });

            let compounded_liquidity_balance = pool_attribute.account_balance;
            if compounded_liquidity_balance != 0 {
                let liquidity_balance_eth = U256::from(oracle_price)
                    .mul(U256::from(compounded_liquidity_balance))
                    .div(U256::from(PRICE_PRECISION));
                total_collateral_in_base_currency =
                    total_collateral_in_base_currency.add(liquidity_balance_eth);
                avg_ltv = avg_ltv.add(liquidity_balance_eth.mul(U256::from(ltv)));
                avg_liquidation_threshold = avg_liquidation_threshold.add(
                    liquidity_balance_eth.mul(U256::from(pool_attribute.liquidation_threshold)),
                );
            }

            let borrow_balance_stored = pool_attribute.account_borrow_balance;
            if borrow_balance_stored != 0 {
                let borrow_balance_eth = U256::from(oracle_price)
                    .mul(U256::from(borrow_balance_stored))
                    .div(U256::from(PRICE_PRECISION));
                total_debt_in_base_currency = total_debt_in_base_currency.add(borrow_balance_eth);
            }

            (
                oracle_price,
                pool_attribute.liquidation_threshold,
                attr_pool,
            )
        } else {
            (0, 0, caller)
        };

        // NOTE: Do not use account_assets as it makes doubled cross-contract calling leads to high gas.
        for asset in self._markets() {
            if asset == skip_pool {
                continue
            }
            // Read the balances and exchange rate from the pool
            let (compounded_liquidity_balance, borrow_balance_stored, exchange_rate_mantissa) =
                PoolRef::get_account_snapshot(&asset, account);

            // If user didn't make any action.
            if compounded_liquidity_balance == 0 && borrow_balance_stored == 0 {
                // If it is modifying token, add to asset_params.
                if token_modify.is_none() || token_modify != Some(asset) {
                    continue
                }
            }

            // Get Metadata of pool
            let PoolMetaData {
                decimals,
                liquidation_threshold,
                underlying,
            } = PoolRef::metadata(&asset);
            let pool_underlying = underlying.ok_or(Error::UnderlyingIsNotSet)?;
            // Get the normalized price of the asset
            let oracle_price: u128 =
                PriceOracleRef::get_price(&oracle, pool_underlying).ok_or(Error::PriceError)?;
            if oracle_price == 0 {
                return Err(Error::PriceError)
            }
            let oracle_price_mantissa = Exp {
                mantissa: WrappedU256::from(U256::from(oracle_price)),
            };

            let collateral_factor_mantissa = self
                ._collateral_factor_mantissa(asset)
                .ok_or(Error::InvalidCollateralFactor)?;

            // Store data for input to calculate the available capacity
            asset_params.push(HypotheticalAccountLiquidityCalculationParam {
                asset,
                decimals,
                token_balance: compounded_liquidity_balance,
                borrow_balance: borrow_balance_stored,
                exchange_rate_mantissa: Exp {
                    mantissa: WrappedU256::from(exchange_rate_mantissa),
                },
                collateral_factor_mantissa: Exp {
                    mantissa: collateral_factor_mantissa,
                },
                oracle_price_mantissa: oracle_price_mantissa.clone(),
            });

            // Calculate data for input to calculate the capacity of balance reduction with liquidation threshold
            let ltv = U256::from(collateral_factor_mantissa);

            if compounded_liquidity_balance != 0 {
                let liquidity_balance_eth = U256::from(oracle_price)
                    .mul(U256::from(compounded_liquidity_balance))
                    .div(U256::from(PRICE_PRECISION));
                total_collateral_in_base_currency =
                    total_collateral_in_base_currency.add(liquidity_balance_eth);
                avg_ltv = avg_ltv.add(liquidity_balance_eth.mul(U256::from(ltv)));
                avg_liquidation_threshold = avg_liquidation_threshold
                    .add(liquidity_balance_eth.mul(U256::from(liquidation_threshold)));
            }

            if borrow_balance_stored != 0 {
                let borrow_balance_eth = U256::from(oracle_price)
                    .mul(U256::from(borrow_balance_stored))
                    .div(U256::from(PRICE_PRECISION));
                total_debt_in_base_currency = total_debt_in_base_currency.add(borrow_balance_eth);
            }
        }

        (avg_ltv, avg_liquidation_threshold) = if total_collateral_in_base_currency.is_zero() {
            (U256::from(0), U256::from(0))
        } else {
            (
                avg_ltv.div(total_collateral_in_base_currency),
                avg_liquidation_threshold.div(total_collateral_in_base_currency),
            )
        };

        let health_factor = calculate_health_factor_from_balances(
            total_collateral_in_base_currency,
            total_debt_in_base_currency,
            avg_liquidation_threshold,
        );

        Ok((
            AccountCollateralData {
                total_collateral_in_base_currency,
                total_debt_in_base_currency,
                avg_ltv,
                avg_liquidation_threshold,
                health_factor,
                asset_price,
                liquidation_threshold,
            },
            asset_params,
        ))
    }

    default fn _balance_decrease_allowed(
        &self,
        pool_attributes: PoolAttributes,
        account: AccountId,
        amount: Balance,
    ) -> Result<()> {
        let oracle = self._oracle().ok_or(Error::OracleIsNotSet)?;

        let (account_data, _) =
            self._calculate_user_account_data(account, Some(pool_attributes.clone()), None)?;

        let total_debt_in_base_currency = account_data.total_debt_in_base_currency;

        if total_debt_in_base_currency.is_zero() {
            return Ok(())
        }

        let underlying = pool_attributes
            .underlying
            .ok_or(Error::UnderlyingIsNotSet)?;

        let asset_price: u128 =
            PriceOracleRef::get_price(&oracle, underlying).ok_or(Error::PriceError)?;
        if asset_price == 0 {
            return Err(Error::PriceError)
        }

        let result = balance_decrease_allowed(BalanceDecreaseAllowedParam {
            total_collateral_in_base_currency: account_data.total_collateral_in_base_currency,
            total_debt_in_base_currency,
            avg_liquidation_threshold: account_data.avg_liquidation_threshold,
            amount_in_base_currency_unit: amount.into(),
            asset_price: asset_price.into(),
            liquidation_threshold: pool_attributes.liquidation_threshold.into(),
        });
        if result {
            return Ok(())
        }
        Err(Error::BalanceDecreaseNotAllowed)
    }

    default fn _emit_market_listed_event(&self, _pool: AccountId) {}

    default fn _emit_new_collateral_factor_event(
        &self,
        _pool: AccountId,
        _old: WrappedU256,
        _new: WrappedU256,
    ) {
    }

    default fn _emit_pool_action_paused_event(
        &self,
        _pool: AccountId,
        _action: String,
        _paused: bool,
    ) {
    }

    default fn _emit_action_paused_event(&self, _action: String, _paused: bool) {}

    default fn _emit_new_price_oracle_event(
        &self,
        _old: Option<AccountId>,
        _new: Option<AccountId>,
    ) {
    }

    default fn _emit_new_flashloan_gateway_event(
        &self,
        _old: Option<AccountId>,
        _new: Option<AccountId>,
    ) {
    }

    default fn _emit_new_close_factor_event(&self, _old: WrappedU256, _new: WrappedU256) {}

    default fn _emit_new_liquidation_incentive_event(&self, _old: WrappedU256, _new: WrappedU256) {}

    default fn _emit_new_borrow_cap_event(&self, _pool: AccountId, _new: Balance) {}

    default fn _emit_manager_updated_event(&self, _old: AccountId, _new: AccountId) {}
}
