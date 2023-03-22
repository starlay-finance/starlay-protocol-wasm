use super::exp_no_err::{
    exp_scale,
    Exp,
};
pub use crate::traits::{
    controller::*,
    pool::PoolRef,
};
use crate::traits::{
    price_oracle::PriceOracleRef,
    types::WrappedU256,
};
use core::ops::{
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
        ZERO_ADDRESS,
    },
};
use primitive_types::U256;

mod utils;
use self::utils::{
    collateral_factor_max_mantissa,
    get_hypothetical_account_liquidity,
    liquidate_calculate_seize_tokens,
    GetHypotheticalAccountLiquidityInput,
    HypotheticalAccountLiquidityCalculationParam,
    LiquidateCalculateSeizeTokensInput,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub markets: Vec<AccountId>,
    pub collateral_factor_mantissa: Mapping<AccountId, WrappedU256>,
    pub mint_guardian_paused: Mapping<AccountId, bool>,
    pub borrow_guardian_paused: Mapping<AccountId, bool>,
    pub seize_guardian_paused: bool,
    pub transfer_guardian_paused: bool,
    pub oracle: AccountId,
    pub close_factor_mantissa: WrappedU256,
    pub liquidation_incentive_mantissa: WrappedU256,
    pub borrow_caps: Mapping<AccountId, Balance>,
    pub manager: AccountId,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            markets: Default::default(),
            collateral_factor_mantissa: Default::default(),
            mint_guardian_paused: Default::default(),
            borrow_guardian_paused: Default::default(),
            seize_guardian_paused: Default::default(),
            transfer_guardian_paused: Default::default(),
            oracle: ZERO_ADDRESS.into(),
            close_factor_mantissa: WrappedU256::from(U256::zero()),
            liquidation_incentive_mantissa: WrappedU256::from(U256::zero()),
            borrow_caps: Default::default(),
            manager: ZERO_ADDRESS.into(),
        }
    }
}

pub trait Internal {
    fn _mint_allowed(&self, pool: AccountId, minter: AccountId, mint_amount: Balance)
        -> Result<()>;
    fn _mint_verify(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    ) -> Result<()>;
    fn _redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
    ) -> Result<()>;
    fn _redeem_verify(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        redeem_tokens: Balance,
    ) -> Result<()>;
    fn _borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()>;
    fn _borrow_verify(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()>;
    fn _repay_borrow_allowed(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;
    fn _repay_borrow_verify(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        borrower_index: u128,
    ) -> Result<()>;
    fn _liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;
    fn _liquidate_borrow_verify(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        seize_tokens: Balance,
    ) -> Result<()>;
    fn _seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;
    fn _seize_verify(
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
    ) -> Result<()>;
    fn _transfer_verify(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()>;
    fn _liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        exchange_rate_mantissa: WrappedU256,
        repay_amount: Balance,
    ) -> Result<Balance>;
    fn _assert_manager(&self) -> Result<()>;
    fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;
    fn _support_market(&mut self, pool: &AccountId) -> Result<()>;
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

    // view function
    fn _markets(&self) -> Vec<AccountId>;
    fn _collateral_factor_mantissa(&self, pool: AccountId) -> Option<WrappedU256>;
    fn _is_listed(&self, pool: AccountId) -> bool;
    fn _mint_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    fn _borrow_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    fn _seize_guardian_paused(&self) -> bool;
    fn _transfer_guardian_paused(&self) -> bool;
    fn _oracle(&self) -> AccountId;
    fn _close_factor_mantissa(&self) -> WrappedU256;
    fn _liquidation_incentive_mantissa(&self) -> WrappedU256;
    fn _borrow_cap(&self, pool: AccountId) -> Option<Balance>;
    fn _manager(&self) -> AccountId;

    fn _account_assets(&self, account: AccountId) -> Vec<AccountId>;
    fn _get_account_liquidity(&self, account: AccountId) -> (U256, U256);
    fn _get_hypothetical_account_liquidity(
        &self,
        account: AccountId,
        token: AccountId,
        redeem_tokens: Balance,
        borrow_amount: Balance,
    ) -> (U256, U256);

    // event emission
    fn _emit_market_listed_event(&self, pool: AccountId);
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

    default fn mint_verify(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    ) -> Result<()> {
        self._mint_verify(pool, minter, mint_amount, mint_tokens)
    }

    default fn redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
    ) -> Result<()> {
        self._redeem_allowed(pool, redeemer, redeem_amount)
    }

    default fn redeem_verify(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        redeem_tokens: Balance,
    ) -> Result<()> {
        self._redeem_verify(pool, redeemer, redeem_amount, redeem_tokens)
    }

    default fn borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()> {
        self._borrow_allowed(pool, borrower, borrow_amount)
    }

    default fn borrow_verify(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()> {
        self._borrow_verify(pool, borrower, borrow_amount)
    }

    default fn repay_borrow_allowed(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        self._repay_borrow_allowed(pool, payer, borrower, repay_amount)
    }

    default fn repay_borrow_verify(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        borrower_index: u128,
    ) -> Result<()> {
        self._repay_borrow_verify(pool, payer, borrower, repay_amount, borrower_index)
    }

    default fn liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        self._liquidate_borrow_allowed(
            pool_borrowed,
            pool_collateral,
            liquidator,
            borrower,
            repay_amount,
        )
    }

    default fn liquidate_borrow_verify(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        seize_tokens: Balance,
    ) -> Result<()> {
        self._liquidate_borrow_verify(
            pool_borrowed,
            pool_collateral,
            liquidator,
            borrower,
            repay_amount,
            seize_tokens,
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

    default fn seize_verify(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()> {
        self._seize_verify(
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
    ) -> Result<()> {
        self._transfer_allowed(pool, src, dst, transfer_tokens)
    }

    default fn transfer_verify(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()> {
        self._transfer_verify(pool, src, dst, transfer_tokens)
    }

    default fn liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        exchange_rate_mantissa: WrappedU256,
        repay_amount: Balance,
    ) -> Result<Balance> {
        self._liquidate_calculate_seize_tokens(
            pool_borrowed,
            pool_collateral,
            exchange_rate_mantissa,
            repay_amount,
        )
    }

    default fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        self._assert_manager()?;
        self._set_price_oracle(new_oracle)
    }

    default fn support_market(&mut self, pool: AccountId) -> Result<()> {
        self._assert_manager()?;
        self._support_market(&pool)?;
        self._emit_market_listed_event(pool);
        Ok(())
    }

    default fn set_collateral_factor_mantissa(
        &mut self,
        pool: AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        self._set_collateral_factor_mantissa(&pool, new_collateral_factor_mantissa)
    }

    default fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_mint_guardian_paused(&pool, paused)
    }

    default fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_borrow_guardian_paused(&pool, paused)
    }

    default fn set_seize_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_seize_guardian_paused(paused)
    }

    default fn set_transfer_guardian_paused(&mut self, paused: bool) -> Result<()> {
        self._assert_manager()?;
        self._set_transfer_guardian_paused(paused)
    }

    default fn set_close_factor_mantissa(
        &mut self,
        new_close_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        self._set_close_factor_mantissa(new_close_factor_mantissa)
    }

    default fn set_liquidation_incentive_mantissa(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        self._assert_manager()?;
        self._set_liquidation_incentive_mantissa(new_liquidation_incentive_mantissa)
    }

    default fn set_borrow_cap(&mut self, pool: AccountId, new_cap: Balance) -> Result<()> {
        self._assert_manager()?;
        self._set_borrow_cap(&pool, new_cap)
    }

    default fn markets(&self) -> Vec<AccountId> {
        self._markets()
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
    default fn oracle(&self) -> AccountId {
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
    default fn manager(&self) -> AccountId {
        self._manager()
    }
    default fn is_listed(&self, pool: AccountId) -> bool {
        self._is_listed(pool)
    }
    default fn account_assets(&self, account: AccountId) -> Vec<AccountId> {
        self._account_assets(account)
    }
    default fn get_account_liquidity(&self, account: AccountId) -> (U256, U256) {
        self._get_account_liquidity(account)
    }
    default fn get_hypothetical_account_liquidity(
        &self,
        account: AccountId,
        token: AccountId,
        redeem_tokens: Balance,
        borrow_amount: Balance,
    ) -> (U256, U256) {
        self._get_hypothetical_account_liquidity(account, token, redeem_tokens, borrow_amount)
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
    default fn _mint_verify(
        &self,
        _pool: AccountId,
        _minter: AccountId,
        _mint_amount: Balance,
        _mint_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
    ) -> Result<()> {
        let (_, shortfall) =
            self._get_hypothetical_account_liquidity(redeemer, pool, redeem_amount, 0);
        if !shortfall.is_zero() {
            return Err(Error::InsufficientLiquidity)
        }

        // FEATURE: update governance token supply index & distribute

        Ok(())
    }
    default fn _redeem_verify(
        &self,
        _pool: AccountId,
        _redeemer: AccountId,
        _redeem_amount: Balance,
        _redeem_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()> {
        if let Some(true) | None = self._borrow_guardian_paused(pool) {
            return Err(Error::BorrowIsPaused)
        }

        // TODO: assertion check - check oracle price for underlying asset

        let borrow_cap = self._borrow_cap(pool).unwrap();
        // borrow cap of 0 corresponds to unlimited borrowing
        if borrow_cap != 0 {
            let total_borrow = PoolRef::total_borrows(&pool);
            if total_borrow > borrow_cap - borrow_amount {
                return Err(Error::BorrowCapReached)
            }
        }

        let (_, shortfall) =
            self._get_hypothetical_account_liquidity(borrower, pool, 0, borrow_amount);
        if !shortfall.is_zero() {
            return Err(Error::InsufficientLiquidity)
        }

        // FEATURE: update governance token borrow index & distribute

        Ok(())
    }
    default fn _borrow_verify(
        &self,
        _pool: AccountId,
        _borrower: AccountId,
        _borrow_amount: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _repay_borrow_allowed(
        &self,
        _pool: AccountId,
        _payer: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
    ) -> Result<()> {
        // FEATURE: update governance token borrow index & distribute

        Ok(())
    }
    default fn _repay_borrow_verify(
        &self,
        _pool: AccountId,
        _payer: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
        _borrower_index: u128,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        _liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        if !self._is_listed(pool_borrowed) || !self._is_listed(pool_collateral) {
            return Err(Error::MarketNotListed)
        }

        // The borrower must have shortfall in order to be liquidatable
        let (_, shortfall) = self._get_account_liquidity(borrower);
        if shortfall.is_zero() {
            return Err(Error::InsufficientShortfall)
        }

        // The liquidator may not repay more than what is allowed by the closeFactor
        let bollow_balance = PoolRef::borrow_balance_stored(&pool_borrowed, borrower);
        let max_close = Exp {
            mantissa: self._close_factor_mantissa(),
        }
        .mul_scalar_truncate(U256::from(bollow_balance));
        if U256::from(repay_amount).gt(&max_close) {
            return Err(Error::TooMuchRepay)
        }

        Ok(())
    }
    default fn _liquidate_borrow_verify(
        &self,
        _pool_borrowed: AccountId,
        _pool_collateral: AccountId,
        _liquidator: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
        _seize_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
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
        let p_collateral_ctrler = PoolRef::controller(&pool_collateral);
        let p_borrowed_ctrler = PoolRef::controller(&pool_borrowed);
        if p_collateral_ctrler != p_borrowed_ctrler {
            return Err(Error::ControllerMismatch)
        }

        // FEATURE: update governance token supply index & distribute to borrower,liquidator

        Ok(())
    }
    default fn _seize_verify(
        &self,
        _pool_collateral: AccountId,
        _pool_borrowed: AccountId,
        _liquidator: AccountId,
        _borrower: AccountId,
        _seize_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _transfer_allowed(
        &self,
        _pool: AccountId,
        _src: AccountId,
        _dst: AccountId,
        _transfer_tokens: Balance,
    ) -> Result<()> {
        if self._transfer_guardian_paused() {
            return Err(Error::TransferIsPaused)
        }

        todo!()
    }
    default fn _transfer_verify(
        &self,
        _pool: AccountId,
        _src: AccountId,
        _dst: AccountId,
        _transfer_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _liquidate_calculate_seize_tokens(
        &self,
        _pool_borrowed: AccountId,
        _pool_collateral: AccountId,
        _exchange_rate_mantissa: WrappedU256,
        _repay_amount: Balance,
    ) -> Result<Balance> {
        let (price_borrowed_mantissa, price_collateral_mantissa) =
            (U256::one().mul(exp_scale()), U256::one().mul(exp_scale())); // TODO
        liquidate_calculate_seize_tokens(&LiquidateCalculateSeizeTokensInput {
            actual_repay_amount: _repay_amount,
            exchange_rate_mantissa: _exchange_rate_mantissa.into(),
            liquidation_incentive_mantissa: self._liquidation_incentive_mantissa().into(),
            price_borrowed_mantissa,
            price_collateral_mantissa,
        })
    }
    default fn _assert_manager(&self) -> Result<()> {
        if Self::env().caller() != self._manager() {
            return Err(Error::CallerIsNotManager)
        }
        Ok(())
    }
    default fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        self.data().oracle = new_oracle;
        Ok(())
    }
    default fn _support_market(&mut self, pool: &AccountId) -> Result<()> {
        self.data().markets.push(*pool);

        // set default states
        self._set_mint_guardian_paused(pool, false)?;
        self._set_collateral_factor_mantissa(pool, WrappedU256::from(0))?;
        self._set_borrow_guardian_paused(pool, false)?;
        self._set_borrow_cap(pool, 0)?;

        Ok(())
    }
    default fn _set_collateral_factor_mantissa(
        &mut self,
        pool: &AccountId,
        new_collateral_factor_mantissa: WrappedU256,
    ) -> Result<()> {
        if U256::from(new_collateral_factor_mantissa).gt(&collateral_factor_max_mantissa()) {
            return Err(Error::InvalidCollateralFactor)
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

    default fn _markets(&self) -> Vec<AccountId> {
        self.data().markets.clone()
    }
    default fn _is_listed(&self, pool: AccountId) -> bool {
        let markets = self._markets();
        for market in markets {
            if market == pool {
                return true
            }
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
    default fn _oracle(&self) -> AccountId {
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
    default fn _manager(&self) -> AccountId {
        self.data().manager
    }

    default fn _account_assets(&self, account: AccountId) -> Vec<AccountId> {
        let mut account_assets = Vec::<AccountId>::new();
        let markets = self._markets();
        for pool in markets {
            let (balance, borrowed, _) = PoolRef::get_account_snapshot(&pool, account);

            // whether deposits or loans exist
            if balance > 0 || borrowed > 0 {
                account_assets.push(pool);
            }
        }
        return account_assets
    }

    default fn _get_account_liquidity(&self, account: AccountId) -> (U256, U256) {
        self._get_hypothetical_account_liquidity(account, ZERO_ADDRESS.into(), 0, 0)
    }

    default fn _get_hypothetical_account_liquidity(
        &self,
        account: AccountId,
        token_modify: AccountId,
        redeem_tokens: Balance,
        borrow_amount: Balance,
    ) -> (U256, U256) {
        // For each asset the account is in
        let account_assets = self._account_assets(account);
        let mut asset_params = Vec::<HypotheticalAccountLiquidityCalculationParam>::new();

        // Prepare parameters for calculation
        for asset in &account_assets {
            // Read the balances and exchange rate from the pool
            let (token_balance, borrow_balance, exchange_rate_mantissa) =
                PoolRef::get_account_snapshot(asset, account);

            // Get the normalized price of the asset
            let oracle_price = Exp {
                mantissa: WrappedU256::from(U256::from(
                    PriceOracleRef::get_underlying_price(&self._oracle(), *asset).unwrap(),
                )),
            };

            asset_params.push(HypotheticalAccountLiquidityCalculationParam {
                asset: *asset,
                token_balance,
                borrow_balance,
                exchange_rate_mantissa: Exp {
                    mantissa: WrappedU256::from(exchange_rate_mantissa),
                },
                collateral_factor_mantissa: Exp {
                    mantissa: self._collateral_factor_mantissa(*asset).unwrap(),
                },
                oracle_price_mantissa: oracle_price.clone(),
            });
        }

        let (sum_collateral, sum_borrow_plus_effect) =
            get_hypothetical_account_liquidity(GetHypotheticalAccountLiquidityInput {
                asset_params,
                token_modify,
                redeem_tokens,
                borrow_amount,
            });

        // These are safe, as the underflow condition is checked first
        if sum_collateral > sum_borrow_plus_effect {
            return (sum_collateral.sub(sum_borrow_plus_effect), U256::from(0))
        } else {
            return (U256::from(0), sum_borrow_plus_effect.sub(sum_collateral))
        }
    }

    default fn _emit_market_listed_event(&self, _pool: AccountId) {}
}
