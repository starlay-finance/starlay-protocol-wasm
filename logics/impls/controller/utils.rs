pub use crate::traits::controller::*;
use crate::{
    impls::exp_no_err::{
        exp_scale,
        Exp,
    },
    traits::types::WrappedU256,
};
use core::ops::{
    Div,
    Mul,
};
use openbrush::traits::Balance;
use primitive_types::U256;

pub struct LiquidateCalculateSeizeTokensInput {
    pub price_borrowed_mantissa: U256,
    pub price_collateral_mantissa: U256,
    pub exchange_rate_mantissa: U256,
    pub liquidation_incentive_mantissa: U256,
    pub actual_repay_amount: Balance,
}

pub fn liquidate_calculate_seize_tokens(
    input: &LiquidateCalculateSeizeTokensInput,
) -> Result<Balance> {
    if input.price_borrowed_mantissa.is_zero() || input.price_collateral_mantissa.is_zero() {
        return Err(Error::PriceError)
    }
    let numerator = Exp {
        mantissa: WrappedU256::from(input.liquidation_incentive_mantissa),
    }
    .mul(Exp {
        mantissa: WrappedU256::from(input.price_borrowed_mantissa),
    });
    let denominator = Exp {
        mantissa: WrappedU256::from(input.price_collateral_mantissa),
    }
    .mul(Exp {
        mantissa: WrappedU256::from(input.exchange_rate_mantissa),
    });
    let ratio = numerator.div(denominator);
    let seize_tokens = ratio.mul_scalar_truncate(U256::from(input.actual_repay_amount));
    Ok(seize_tokens.as_u128())
}

pub fn collateral_factor_max_mantissa() -> U256 {
    // 90%
    exp_scale().mul(U256::from(90)).div(U256::from(100))
}

#[cfg(test)]
mod tests {
    use core::ops::{
        Div,
        Mul,
    };

    use crate::impls::exp_no_err::exp_scale;

    use super::*;
    use primitive_types::U256;
    fn mts(val: u128) -> U256 {
        U256::from(val).mul(exp_scale())
    }
    #[test]
    fn test_liquidate_calculate_seize_tokens_price_is_zero() {
        struct Case<'a> {
            input: &'a LiquidateCalculateSeizeTokensInput,
            want_err: Error,
        }
        let cases: &[Case] = &[
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: U256::one(),
                    price_collateral_mantissa: U256::zero(),
                    exchange_rate_mantissa: U256::one(),
                    liquidation_incentive_mantissa: U256::one(),
                    actual_repay_amount: 1,
                },
                want_err: Error::PriceError,
            },
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: U256::zero(),
                    price_collateral_mantissa: U256::one(),
                    exchange_rate_mantissa: U256::one(),
                    liquidation_incentive_mantissa: U256::one(),
                    actual_repay_amount: 1,
                },
                want_err: Error::PriceError,
            },
        ];
        for case in cases {
            let result = liquidate_calculate_seize_tokens(case.input.into());
            assert_eq!(result.err().unwrap(), case.want_err);
        }
    }
    #[test]
    fn test_liquidate_calculate_seize_tokens() {
        struct Case<'a> {
            input: &'a LiquidateCalculateSeizeTokensInput,
        }
        let cases: &[Case] = &[
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: mts(100),
                    price_collateral_mantissa: mts(200),
                    exchange_rate_mantissa: mts(10).div(U256::from(100)),
                    liquidation_incentive_mantissa: mts(10).div(U256::from(100)),
                    actual_repay_amount: 1,
                },
            },
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: mts(233),
                    price_collateral_mantissa: mts(957),
                    exchange_rate_mantissa: mts(20).div(U256::from(100)),
                    liquidation_incentive_mantissa: mts(10).div(U256::from(100)),
                    actual_repay_amount: 123,
                },
            },
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: mts(99827),
                    price_collateral_mantissa: mts(99823),
                    exchange_rate_mantissa: mts(23).div(U256::from(100)),
                    liquidation_incentive_mantissa: mts(11).div(U256::from(100)),
                    actual_repay_amount: 1237,
                },
            },
        ];
        for case in cases {
            let got = liquidate_calculate_seize_tokens(case.input);
            //  seize_amount = actual_repay_amount * liquidation_incentive * price_borrowed / price_collateral

            //  seize_tokens = seize_amount / exchange_rate
            //   = actual_repay_amount * (liquidation_incentive * price_borrowed) / (price_collateral * exchange_rate)
            let input = case.input;
            let want = U256::from(input.actual_repay_amount)
                .mul(input.liquidation_incentive_mantissa)
                .mul(input.price_borrowed_mantissa)
                .div(input.price_collateral_mantissa)
                .div(input.exchange_rate_mantissa);
            assert_eq!(got.unwrap(), want.as_u128());
        }
    }
}
