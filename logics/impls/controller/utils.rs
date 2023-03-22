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

pub fn get_hypothetical_account_liquidity_per_asset(
    token_balance: Balance,
    borrow_balance: Balance,
    exchange_rate_mantissa: Exp,
    collateral_factor_mantissa: Exp,
    oracle_price_mantissa: Exp,
) -> (Exp, U256, U256) {
    // Pre-compute a conversion factor from tokens -> base token (normalized price value)
    let token_to_denom = collateral_factor_mantissa
        .mul(exchange_rate_mantissa)
        .mul(oracle_price_mantissa.clone());
    // sumCollateral += tokensToDenom * cTokenBalance
    let collateral = token_to_denom
        .clone()
        .mul_scalar_truncate(U256::from(token_balance));
    // sumBorrowPlusEffects += oraclePrice * borrowBalance
    let borrow_plus_effect = oracle_price_mantissa
        .clone()
        .mul_scalar_truncate(U256::from(borrow_balance));

    return (token_to_denom, collateral, borrow_plus_effect)
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

    #[test]
    fn test_get_hypothetical_account_liqudity_per_asset() {
        struct Case {
            input: Input,
            expected: Expected,
        }
        struct Input {
            token_balance: Balance,
            borrow_balance: Balance,
            exchange_rate_mantissa: u128,
            collateral_factor_mantissa: u128,
            oracle_price_mantissa: u128,
        }
        struct Expected {
            collateral: u128,
            borrow_plus_effect: u128,
        }
        let mantissa = 10_u128.pow(18);
        let cases = vec![
            Case {
                input: Input {
                    token_balance: 200,
                    borrow_balance: 100,
                    exchange_rate_mantissa: mantissa * 1,
                    collateral_factor_mantissa: mantissa * 50 / 100, // 50%
                    oracle_price_mantissa: mantissa * 1,
                },
                expected: Expected {
                    collateral: 100,
                    borrow_plus_effect: 100,
                },
            }, // simple
            Case {
                input: Input {
                    token_balance: 111111,
                    borrow_balance: 100000,
                    exchange_rate_mantissa: mantissa * 1,
                    collateral_factor_mantissa: mantissa * 90 / 100, // 90%
                    oracle_price_mantissa: mantissa * 100,
                },
                expected: Expected {
                    collateral: 9999990,
                    borrow_plus_effect: 10000000,
                },
            }, // HF = almost 100%
            Case {
                input: Input {
                    token_balance: 1000,
                    borrow_balance: 1000,
                    exchange_rate_mantissa: mantissa * 5 / 10, // 0.5
                    collateral_factor_mantissa: mantissa * 25 / 100, // 25%
                    oracle_price_mantissa: mantissa * 100,
                },
                expected: Expected {
                    collateral: 12500,
                    borrow_plus_effect: 100000,
                },
            }, // low exchange_rate, collateral_factor (HF = 12.5%)
        ];
        for case in cases {
            let (_, collateral, borrow_plus_effect) = get_hypothetical_account_liquidity_per_asset(
                case.input.token_balance,
                case.input.borrow_balance,
                Exp {
                    mantissa: WrappedU256::from(U256::from(case.input.exchange_rate_mantissa)),
                },
                Exp {
                    mantissa: WrappedU256::from(U256::from(case.input.collateral_factor_mantissa)),
                },
                Exp {
                    mantissa: WrappedU256::from(U256::from(case.input.oracle_price_mantissa)),
                },
            );
            assert_eq!(collateral, U256::from(case.expected.collateral));
            assert_eq!(
                borrow_plus_effect,
                U256::from(case.expected.borrow_plus_effect)
            );
        }
    }
}
