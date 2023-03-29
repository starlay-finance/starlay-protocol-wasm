pub use crate::traits::controller::*;
use crate::{
    impls::exp_no_err::{
        exp_scale,
        Exp,
    },
    traits::types::WrappedU256,
};
use core::ops::{
    Add,
    Div,
    Mul,
};
use ink::prelude::vec::Vec;
use openbrush::traits::{
    AccountId,
    Balance,
};
use primitive_types::U256;

pub struct LiquidateCalculateSeizeTokensInput {
    pub price_borrowed_mantissa: U256,
    pub price_collateral_mantissa: U256,
    pub exchange_rate_mantissa: U256,
    pub liquidation_incentive_mantissa: U256,
    pub actual_repay_amount: Balance,
}

pub fn liquidate_calculate_seize_tokens(input: &LiquidateCalculateSeizeTokensInput) -> Balance {
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
    seize_tokens.as_u128()
}

pub fn collateral_factor_max_mantissa() -> U256 {
    // 90%
    exp_scale().mul(U256::from(90)).div(U256::from(100))
}

#[derive(Debug)]
pub struct GetHypotheticalAccountLiquidityInput {
    pub asset_params: Vec<HypotheticalAccountLiquidityCalculationParam>,
    pub token_modify: AccountId,
    pub redeem_tokens: Balance,
    pub borrow_amount: Balance,
}
#[derive(Clone, Debug)]
pub struct HypotheticalAccountLiquidityCalculationParam {
    pub asset: AccountId,
    pub decimals: u8,
    pub token_balance: Balance,
    pub borrow_balance: Balance,
    pub exchange_rate_mantissa: Exp,
    pub collateral_factor_mantissa: Exp,
    pub oracle_price_mantissa: Exp,
}
pub fn get_hypothetical_account_liquidity(
    input: GetHypotheticalAccountLiquidityInput,
) -> (U256, U256) {
    let mut sum_collateral = U256::from(0);
    let mut sum_borrow_plus_effect = U256::from(0);

    let GetHypotheticalAccountLiquidityInput {
        asset_params,
        token_modify,
        redeem_tokens,
        borrow_amount,
    } = input;

    for param in asset_params {
        let (token_to_denom, collateral, borrow_plus_effect) =
            get_hypothetical_account_liquidity_per_asset(
                param.token_balance,
                param.borrow_balance,
                param.decimals,
                param.exchange_rate_mantissa.clone(),
                param.collateral_factor_mantissa.clone(),
                param.oracle_price_mantissa.clone(),
            );

        sum_collateral = sum_collateral.add(collateral);
        sum_borrow_plus_effect = sum_borrow_plus_effect.add(borrow_plus_effect);

        // Calculate effects of interacting with cTokenModify
        if param.asset == token_modify {
            let to_flatten = |volume: U256| {
                volume
                    .mul(exp_scale())
                    .div(U256::from(10_u128.pow(param.decimals.into())))
            };
            // redeem effect
            // sumBorrowPlusEffects += tokensToDenom * redeemTokens
            sum_borrow_plus_effect = token_to_denom.clone().mul_scalar_truncate_add_uint(
                to_flatten(U256::from(redeem_tokens)),
                sum_borrow_plus_effect,
            );

            // borrow effect
            // sumBorrowPlusEffects += oraclePrice * borrowAmount
            sum_borrow_plus_effect = param
                .oracle_price_mantissa
                .clone()
                .mul_scalar_truncate_add_uint(
                    to_flatten(U256::from(borrow_amount)),
                    sum_borrow_plus_effect,
                );
        }
    }

    (sum_collateral, sum_borrow_plus_effect)
}

pub fn get_hypothetical_account_liquidity_per_asset(
    token_balance: Balance,
    borrow_balance: Balance,
    decimals: u8,
    exchange_rate_mantissa: Exp,
    collateral_factor_mantissa: Exp,
    oracle_price_mantissa: Exp,
) -> (Exp, U256, U256) {
    // Pre-compute a conversion factor from tokens -> base token (normalized price value)
    let token_to_denom = collateral_factor_mantissa
        .mul(exchange_rate_mantissa)
        .mul(oracle_price_mantissa.clone());

    let to_flatten = |volume: U256| {
        volume
            .mul(exp_scale())
            .div(U256::from(10_u128.pow(decimals.into())))
    };
    // sumCollateral += tokensToDenom * cTokenBalance
    let collateral = token_to_denom
        .clone()
        .mul_scalar_truncate(U256::from(token_balance));
    let flatten_collateral = to_flatten(collateral);
    // sumBorrowPlusEffects += oraclePrice * borrowBalance
    let borrow_plus_effect = oracle_price_mantissa
        .clone()
        .mul_scalar_truncate(U256::from(borrow_balance));
    let flatten_borrow_plus_effect = to_flatten(borrow_plus_effect);

    return (
        token_to_denom,
        flatten_collateral,
        flatten_borrow_plus_effect,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impls::exp_no_err::exp_scale;
    use core::ops::{
        Div,
        Mul,
    };
    use openbrush::traits::ZERO_ADDRESS;
    use primitive_types::U256;
    fn mts(val: u128) -> U256 {
        U256::from(val).mul(exp_scale())
    }
    // TODO: in controller contract
    // #[test]
    // fn test_liquidate_calculate_seize_tokens_price_is_zero() {
    //     struct Case<'a> {
    //         input: &'a LiquidateCalculateSeizeTokensInput,
    //         want_err: Error,
    //     }
    //     let cases: &[Case] = &[
    //         Case {
    //             input: &LiquidateCalculateSeizeTokensInput {
    //                 price_borrowed_mantissa: U256::one(),
    //                 price_collateral_mantissa: U256::zero(),
    //                 exchange_rate_mantissa: U256::one(),
    //                 liquidation_incentive_mantissa: U256::one(),
    //                 actual_repay_amount: 1,
    //             },
    //             want_err: Error::PriceError,
    //         },
    //         Case {
    //             input: &LiquidateCalculateSeizeTokensInput {
    //                 price_borrowed_mantissa: U256::zero(),
    //                 price_collateral_mantissa: U256::one(),
    //                 exchange_rate_mantissa: U256::one(),
    //                 liquidation_incentive_mantissa: U256::one(),
    //                 actual_repay_amount: 1,
    //             },
    //             want_err: Error::PriceError,
    //         },
    //     ];
    //     for case in cases {
    //         let result = liquidate_calculate_seize_tokens(case.input.into());
    //         assert_eq!(result.err().unwrap(), case.want_err);
    //     }
    // }
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
            assert_eq!(got, want.as_u128());
        }
    }

    #[test]
    fn test_get_hypothetical_account_liquidity_per_asset() {
        let mantissa = 10_u128.pow(18);
        let pow10_6 = 10_u128.pow(6);
        let pow10_18 = 10_u128.pow(18);

        struct Case {
            input: Input,
            expected: Expected,
        }
        struct Input {
            token_balance: Balance,
            borrow_balance: Balance,
            decimals: u8,
            exchange_rate_mantissa: u128,
            collateral_factor_mantissa: u128,
            oracle_price_mantissa: u128,
        }
        struct Expected {
            collateral: u128,
            borrow_plus_effect: u128,
        }
        let cases = vec![
            Case {
                input: Input {
                    token_balance: 200 * pow10_6,
                    borrow_balance: 100 * pow10_6,
                    decimals: 6,
                    exchange_rate_mantissa: mantissa * 1,
                    collateral_factor_mantissa: mantissa * 50 / 100, // 50%
                    oracle_price_mantissa: mantissa * 1,
                },
                expected: Expected {
                    collateral: 100 * mantissa,
                    borrow_plus_effect: 100 * mantissa,
                },
            }, // simple
            Case {
                input: Input {
                    token_balance: 111111 * pow10_6,
                    borrow_balance: 100000 * pow10_6,
                    decimals: 6,
                    exchange_rate_mantissa: mantissa * 1,
                    collateral_factor_mantissa: mantissa * 90 / 100, // 90%
                    oracle_price_mantissa: mantissa * 100,
                },
                expected: Expected {
                    collateral: 9999990 * mantissa,
                    borrow_plus_effect: 10000000 * mantissa,
                },
            }, // HF = almost 100%
            Case {
                input: Input {
                    token_balance: 1000 * pow10_18,
                    borrow_balance: 1000 * pow10_18,
                    decimals: 18,
                    exchange_rate_mantissa: mantissa * 5 / 10, // 0.5
                    collateral_factor_mantissa: mantissa * 25 / 100, // 25%
                    oracle_price_mantissa: mantissa * 100,
                },
                expected: Expected {
                    collateral: 12500 * mantissa,
                    borrow_plus_effect: 100000 * mantissa,
                },
            }, // low exchange_rate, collateral_factor (HF = 12.5%)
        ];
        for case in cases {
            let (_, collateral, borrow_plus_effect) = get_hypothetical_account_liquidity_per_asset(
                case.input.token_balance,
                case.input.borrow_balance,
                case.input.decimals,
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

    #[test]
    fn test_get_hypothetical_account_liquidity_without_borrows() {
        let mantissa = 10_u128.pow(18);
        let pow10_6 = 10_u128.pow(6);
        let pow10_18 = 10_u128.pow(18);
        let to_exp = |val: u128| {
            Exp {
                mantissa: WrappedU256::from(U256::from(val)),
            }
        };

        struct Case {
            input: GetHypotheticalAccountLiquidityInput,
            expected: Expected,
        }
        struct Expected {
            sum_collateral: u128,
            sum_borrow_plus_effect: u128,
        }

        // decimal 6 token with 90% collateral factor
        let token1_dec6_param = HypotheticalAccountLiquidityCalculationParam {
            asset: AccountId::from([1; 32]),
            decimals: 6,
            token_balance: 10_000 * pow10_6,
            borrow_balance: 0,
            exchange_rate_mantissa: to_exp(mantissa * 1),
            collateral_factor_mantissa: to_exp(mantissa * 90 / 100), // 90%
            oracle_price_mantissa: to_exp(mantissa * 1),
        };
        let token1_dec6_param_with_borrow = HypotheticalAccountLiquidityCalculationParam {
            borrow_balance: 5_000 * pow10_6,
            ..token1_dec6_param.clone()
        };
        // decimal 6 token with 50% collateral factor
        let token2_dec6_param = HypotheticalAccountLiquidityCalculationParam {
            asset: AccountId::from([2; 32]),
            decimals: 6,
            token_balance: 50_000 * pow10_6,
            borrow_balance: 0,
            exchange_rate_mantissa: to_exp(mantissa * 1),
            collateral_factor_mantissa: to_exp(mantissa * 50 / 100), // 50%
            oracle_price_mantissa: to_exp(mantissa * 1),
        };
        let token2_dec6_param_with_borrow = HypotheticalAccountLiquidityCalculationParam {
            borrow_balance: 20_000 * pow10_6,
            ..token2_dec6_param.clone()
        };
        // decimal 18 token with 90% collateral factor
        let token3_dec18_param = HypotheticalAccountLiquidityCalculationParam {
            asset: AccountId::from([3; 32]),
            decimals: 18,
            token_balance: 25_000 * pow10_18,
            borrow_balance: 0,
            exchange_rate_mantissa: to_exp(mantissa * 1),
            collateral_factor_mantissa: to_exp(mantissa * 90 / 100), // 90%
            oracle_price_mantissa: to_exp(mantissa * 1),
        };
        let token3_dec18_param_with_borrow = HypotheticalAccountLiquidityCalculationParam {
            borrow_balance: 15_000 * pow10_18,
            ..token3_dec18_param.clone()
        };

        let asset_params_without_borrows =
            vec![token1_dec6_param, token2_dec6_param, token3_dec18_param];
        let asset_params_with_borrows = vec![
            token1_dec6_param_with_borrow,
            token2_dec6_param_with_borrow,
            token3_dec18_param_with_borrow,
        ];
        let cases = vec![
            // no redeem & borrow
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_without_borrows.clone(),
                    token_modify: ZERO_ADDRESS.into(),
                    redeem_tokens: 0,
                    borrow_amount: 0,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: 0,
                },
            },
            // some redeem with decimal 6 token with 90% collateral factor
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_without_borrows.clone(),
                    token_modify: AccountId::from([1; 32]),
                    redeem_tokens: 7_500 * pow10_6,
                    borrow_amount: 0,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: (7_500 * 90 / 100) * mantissa,
                },
            },
            // some redeem with decimal 6 token with 50% collateral factor
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_without_borrows.clone(),
                    token_modify: AccountId::from([2; 32]),
                    redeem_tokens: 35_000 * pow10_6,
                    borrow_amount: 0,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: (35_000 * 50 / 100) * mantissa,
                },
            },
            // some borrow with decimal 6 token with 90% collateral factor
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_without_borrows.clone(),
                    token_modify: AccountId::from([1; 32]),
                    redeem_tokens: 0,
                    borrow_amount: 7_500 * pow10_6,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: 7_500 * mantissa,
                },
            },
            // some borrow with decimal 18 token with 90% collateral factor
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_without_borrows.clone(),
                    token_modify: AccountId::from([3; 32]),
                    redeem_tokens: 0,
                    borrow_amount: 15_000 * pow10_18,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: 15_000 * mantissa,
                },
            },
            // (existing some borrows) no redeem/borrow
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_with_borrows.clone(),
                    token_modify: ZERO_ADDRESS.into(),
                    redeem_tokens: 0,
                    borrow_amount: 0,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: (5_000 + 20_000 + 15_000) * mantissa,
                },
            },
            // (existing some borrows) some redeem
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_with_borrows.clone(),
                    token_modify: AccountId::from([1; 32]),
                    redeem_tokens: 7_500 * pow10_6,
                    borrow_amount: 0,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: (5_000 + 20_000 + 15_000 + 7_500 * 90 / 100) * mantissa,
                },
            },
            // (existing some borrows) some borrow
            Case {
                input: GetHypotheticalAccountLiquidityInput {
                    asset_params: asset_params_with_borrows.clone(),
                    token_modify: AccountId::from([3; 32]),
                    redeem_tokens: 0,
                    borrow_amount: 7_500 * pow10_18,
                },
                expected: Expected {
                    sum_collateral: ((10_000 * 90 / 100)
                        + (50_000 * 50 / 100)
                        + (25_000 * 90 / 100))
                        * mantissa,
                    sum_borrow_plus_effect: (5_000 + 20_000 + 15_000 + 7_500) * mantissa,
                },
            },
        ];

        for case in cases {
            let (sum_collateral, sum_borrow_plus_effect) =
                get_hypothetical_account_liquidity(case.input);
            assert_eq!(sum_collateral, U256::from(case.expected.sum_collateral));
            assert_eq!(
                sum_borrow_plus_effect,
                U256::from(case.expected.sum_borrow_plus_effect)
            );
        }
    }
}
