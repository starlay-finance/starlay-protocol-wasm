pub use crate::traits::controller::*;
use crate::{
    impls::{
        exp_no_err::{
            exp_scale,
            Exp,
        },
        percent_math::Percent,
        price_oracle::PRICE_PRECISION,
        wad_ray_math::Wad,
    },
    traits::types::WrappedU256,
};
use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};
use ink::prelude::vec::Vec;
use openbrush::traits::{
    AccountId,
    Balance,
};
use primitive_types::U256;

pub const HEALTH_FACTOR_LIQUIDATION_THRESHOLD: u128 = 10_u128.pow(18);

pub struct LiquidateCalculateSeizeTokensInput {
    pub price_borrowed_mantissa: U256,
    pub decimals_borrowed: u8,
    pub price_collateral_mantissa: U256,
    pub decimals_collateral: u8,
    pub exchange_rate_mantissa: U256,
    pub liquidation_incentive_mantissa: U256,
    pub actual_repay_amount: Balance,
}

/// Calculate seize value when liquidation
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
    let seize_tokens = ratio.mul_scalar_truncate(
        U256::from(input.actual_repay_amount)
            .mul(U256::from(10).pow(input.decimals_collateral.into()))
            .div(U256::from(10).pow(input.decimals_borrowed.into())),
    );
    seize_tokens.as_u128()
}

#[derive(Clone, Debug)]
pub struct BalanceDecreaseAllowedParam {
    pub asset_price: U256,
    pub amount_in_base_currency_unit: U256,
    pub total_collateral_in_base_currency: U256,
    pub avg_liquidation_threshold: U256,
    pub liquidation_threshold: U256,
    pub total_debt_in_base_currency: U256,
}

pub fn balance_decrease_allowed(param: BalanceDecreaseAllowedParam) -> bool {
    let amount_to_decrease_in_base_currency = param
        .asset_price
        .mul(param.amount_in_base_currency_unit)
        .div(U256::from(PRICE_PRECISION));

    if param.total_collateral_in_base_currency < amount_to_decrease_in_base_currency {
        return false
    }

    let collateral_balance_after_decrease = param
        .total_collateral_in_base_currency
        .sub(amount_to_decrease_in_base_currency);

    if collateral_balance_after_decrease.is_zero() {
        return false
    }

    let liquidation_threshold_after_decrease = param
        .total_collateral_in_base_currency
        .mul(param.avg_liquidation_threshold)
        .sub(amount_to_decrease_in_base_currency.mul(param.liquidation_threshold))
        .div(collateral_balance_after_decrease);

    let health_factor_after_decrease = calculate_health_factor_from_balances(
        collateral_balance_after_decrease,
        param.total_debt_in_base_currency,
        liquidation_threshold_after_decrease,
    );

    health_factor_after_decrease >= U256::from(HEALTH_FACTOR_LIQUIDATION_THRESHOLD)
}

/// Maximum value of Collateral Factor
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
/// Calculate the available capacity (hypothetical_account_liquidity) for a given user
/// NOTE: This function has no state and calculates its arguments as source information
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

/// Calculate the available capacity in a pool for a given user
/// NOTE: This function has no state and calculates its arguments as source information
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

/// Calculate Health Factor from Balance
pub fn calculate_health_factor_from_balances(
    total_collateral_in_base_currency: U256,
    total_debt_in_base_currency: U256,
    liquidation_threshold: U256,
) -> U256 {
    if total_debt_in_base_currency.is_zero() {
        return U256::MAX
    }

    let percent_mul_result = (Percent {
        percentage: liquidation_threshold,
    })
    .percent_mul(total_collateral_in_base_currency);

    if percent_mul_result.is_err() {
        return U256::from(0)
    }

    let wad_div_result = (Wad {
        mantissa: WrappedU256::from(percent_mul_result.unwrap()),
    })
    .wad_div(Wad {
        mantissa: WrappedU256::from(total_debt_in_base_currency),
    });

    if wad_div_result.is_err() {
        return U256::from(0)
    }

    U256::from(wad_div_result.unwrap().mantissa)
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
                    decimals_borrowed: 0,
                    price_collateral_mantissa: mts(200),
                    decimals_collateral: 0,
                    exchange_rate_mantissa: mts(10).div(U256::from(100)),
                    liquidation_incentive_mantissa: mts(10).div(U256::from(100)),
                    actual_repay_amount: 1,
                },
            },
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: mts(233),
                    decimals_borrowed: 0,
                    price_collateral_mantissa: mts(957),
                    decimals_collateral: 0,
                    exchange_rate_mantissa: mts(20).div(U256::from(100)),
                    liquidation_incentive_mantissa: mts(10).div(U256::from(100)),
                    actual_repay_amount: 123,
                },
            },
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: mts(99827),
                    decimals_borrowed: 0,
                    price_collateral_mantissa: mts(99823),
                    decimals_collateral: 0,
                    exchange_rate_mantissa: mts(23).div(U256::from(100)),
                    liquidation_incentive_mantissa: mts(11).div(U256::from(100)),
                    actual_repay_amount: 1237,
                },
            },
            Case {
                input: &LiquidateCalculateSeizeTokensInput {
                    price_borrowed_mantissa: mts(1),
                    decimals_borrowed: 18,
                    price_collateral_mantissa: mts(1),
                    decimals_collateral: 6,
                    exchange_rate_mantissa: mts(1),
                    liquidation_incentive_mantissa: mts(108).div(U256::from(100)),
                    actual_repay_amount: 1000 * 10_u128.pow(18),
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
                .div(input.exchange_rate_mantissa)
                .mul(U256::from(10).pow(input.decimals_collateral.into()))
                .div(U256::from(10).pow(input.decimals_borrowed.into()));
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

    #[test]
    fn test_calculate_health_factor_from_balances() {
        struct Case {
            total_collateral_in_base_currency: U256,
            total_debt_in_base_currency: U256,
            liquidation_threshold: U256,
            expected: U256,
            name: &'static str,
        }
        let one: U256 = U256::from(10).pow(U256::from(18));
        let one_percent: U256 = U256::from(100);
        let cases = vec![
            Case {
                name: "debt is zero",
                total_collateral_in_base_currency: 0.into(),
                total_debt_in_base_currency: 0.into(),
                liquidation_threshold: 0.into(),
                expected: U256::MAX,
            },
            Case {
                name: "just hits liquidation threshold",
                total_collateral_in_base_currency: one,
                total_debt_in_base_currency: one.div(U256::from(100)),
                liquidation_threshold: one_percent,
                expected: one,
            },
        ];
        for case in cases {
            let got = calculate_health_factor_from_balances(
                case.total_collateral_in_base_currency,
                case.total_debt_in_base_currency,
                case.liquidation_threshold,
            );
            assert_eq!(got, case.expected, "{}", case.name);
        }
    }
    #[test]
    fn test_balance_decrease_allowed() {
        struct Case {
            input: BalanceDecreaseAllowedParam,
            expected: bool,
            name: &'static str,
        }
        let one: U256 = U256::from(10).pow(U256::from(18));
        let price_one: U256 = one;
        let mantissa_one: U256 = one;
        let one_percent: U256 = U256::from(100);
        let cases = vec![
            Case {
                name: "all params are zero",
                input: BalanceDecreaseAllowedParam {
                    amount_in_base_currency_unit: 0.into(),
                    asset_price: 0.into(),
                    avg_liquidation_threshold: 0.into(),
                    liquidation_threshold: 0.into(),
                    total_collateral_in_base_currency: 0.into(),
                    total_debt_in_base_currency: 0.into(),
                },
                expected: false,
            },
            Case {
                name: "if there is a borrow, there can't be 0 collateral",
                input: BalanceDecreaseAllowedParam {
                    amount_in_base_currency_unit: one,
                    asset_price: price_one,
                    avg_liquidation_threshold: mantissa_one,
                    liquidation_threshold: mantissa_one,
                    total_collateral_in_base_currency: mantissa_one,
                    total_debt_in_base_currency: 0.into(),
                },
                expected: false,
            },
            Case {
                name: "just hit the boundary of liquidation threshold: pass",
                input: BalanceDecreaseAllowedParam {
                    asset_price: price_one,
                    // 80 %
                    avg_liquidation_threshold: one_percent.mul(U256::from(80)),
                    // 80 %
                    liquidation_threshold: one_percent.mul(U256::from(80)),
                    // 110
                    total_collateral_in_base_currency: one.mul(U256::from(110)),
                    // 80
                    total_debt_in_base_currency: one.mul(U256::from(80)),
                    // decrease amount just hits the boundary should be 10
                    amount_in_base_currency_unit: one.mul(U256::from(10)),
                },
                expected: true,
            },
            Case {
                name: "just hit the boundary of liquidation threshold: fail",
                input: BalanceDecreaseAllowedParam {
                    asset_price: price_one,
                    // 80 %
                    avg_liquidation_threshold: one_percent.mul(U256::from(80)),
                    // 80 %
                    liquidation_threshold: one_percent.mul(U256::from(80)),
                    // 110
                    total_collateral_in_base_currency: one.mul(U256::from(110)),
                    // 80
                    total_debt_in_base_currency: one.mul(U256::from(80)),
                    // decrease amount just hits the boundary should be 10 so should fail with 1e19 + 1 + 50(see: wad_div)
                    amount_in_base_currency_unit: one.mul(U256::from(10)).add(U256::from(51)),
                },
                expected: false,
            },
            Case {
                name: "health factor is 2",
                input: BalanceDecreaseAllowedParam {
                    asset_price: price_one,
                    // 80 %
                    avg_liquidation_threshold: one_percent.mul(U256::from(80)),
                    // 80 %
                    liquidation_threshold: one_percent.mul(U256::from(80)),
                    // 210
                    total_collateral_in_base_currency: one.mul(U256::from(210)),
                    // 80
                    total_debt_in_base_currency: one.mul(U256::from(80)),
                    amount_in_base_currency_unit: one.mul(U256::from(10)),
                },
                expected: true,
            },
        ];
        for case in cases {
            assert_eq!(
                balance_decrease_allowed(case.input.clone()),
                case.expected,
                "case: {}",
                case.name
            );
        }
    }
}
