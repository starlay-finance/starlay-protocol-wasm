use openbrush::traits::{
    Balance,
    Timestamp,
};
use primitive_types::U256;

use super::super::exp_no_err::{
    exp_scale,
    Exp,
};
pub use crate::traits::pool::*;
use crate::traits::types::WrappedU256;
use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};

pub fn borrow_rate_max_mantissa() -> U256 {
    // .0005% / time
    exp_scale().mul(U256::from(5)).div(U256::from(1000 * 100))
}

pub fn protocol_seize_share_mantissa() -> U256 {
    exp_scale().mul(U256::from(28)).div(U256::from(10 * 100)) // 2.8%
}

pub struct CalculateInterestInput {
    pub total_borrows: Balance,
    pub total_reserves: Balance,
    pub borrow_index: Exp,
    pub borrow_rate: U256,
    pub old_block_timestamp: Timestamp,
    pub new_block_timestamp: Timestamp,
    pub reserve_factor_mantissa: U256,
}

pub struct CalculateInterestOutput {
    pub borrow_index: Exp,
    pub total_borrows: Balance,
    pub total_reserves: Balance,
    pub interest_accumulated: Balance,
}

pub fn calculate_interest(input: &CalculateInterestInput) -> Result<CalculateInterestOutput> {
    if input.borrow_rate.gt(&borrow_rate_max_mantissa()) {
        return Err(Error::BorrowRateIsAbsurdlyHigh)
    }
    let delta = input
        .new_block_timestamp
        .abs_diff(input.old_block_timestamp);
    let simple_interest_factor = Exp {
        mantissa: WrappedU256::from(input.borrow_rate),
    }
    .mul_mantissa(U256::from(delta));

    let interest_accumulated =
        simple_interest_factor.mul_scalar_truncate(U256::from(input.total_borrows));

    let total_borrows_new = interest_accumulated.as_u128().add(input.total_borrows);
    let total_reserves_new = Exp {
        mantissa: WrappedU256::from(input.reserve_factor_mantissa),
    }
    .mul_scalar_truncate_add_uint(interest_accumulated, U256::from(input.total_reserves));
    let borrow_index_new = simple_interest_factor.mul_scalar_truncate_add_uint(
        input.borrow_index.mantissa.into(),
        input.borrow_index.mantissa.into(),
    );
    Ok(CalculateInterestOutput {
        borrow_index: Exp {
            mantissa: WrappedU256::from(borrow_index_new),
        },
        interest_accumulated: interest_accumulated.as_u128(),
        total_borrows: total_borrows_new,
        total_reserves: total_reserves_new.as_u128(), // TODO
    })
}

// returns liquidator_seize_tokens, protocol_seize_amount and protocol_seize_tokens
pub fn protocol_seize_amount(
    exchange_rate: Exp,
    seize_tokens: Balance,
    protocol_seize_share_mantissa: U256,
) -> (Balance, Balance, Balance) {
    let protocol_seize_tokens = Exp {
        mantissa: WrappedU256::from(U256::from(seize_tokens).mul(protocol_seize_share_mantissa)),
    }
    .truncate();
    let liquidator_seize_tokens = U256::from(seize_tokens).sub(protocol_seize_tokens);
    (
        liquidator_seize_tokens.as_u128(),
        exchange_rate
            .mul_scalar_truncate(protocol_seize_tokens)
            .as_u128(),
        protocol_seize_tokens.as_u128(),
    )
}

pub fn exchange_rate(
    total_supply: Balance,
    total_cash: Balance,
    total_borrows: Balance,
    total_reserves: Balance,
) -> U256 {
    if total_supply == 0 {
        return U256::zero()
    };
    let cash_plus_borrows_minus_reserves = total_cash.add(total_borrows).sub(total_reserves);
    U256::from(cash_plus_borrows_minus_reserves)
        .mul(exp_scale())
        .div(U256::from(total_supply))
}

#[cfg(test)]
mod tests {
    use super::Exp;

    use super::*;
    use primitive_types::U256;
    fn mantissa() -> U256 {
        U256::from(10).pow(U256::from(18))
    }
    #[test]
    fn test_calculate_interest_panic_if_over_borrow_rate_max() {
        let input = CalculateInterestInput {
            borrow_index: Exp {
                mantissa: WrappedU256::from(U256::zero()),
            },
            borrow_rate: U256::one().mul(U256::from(10)).pow(U256::from(18)),
            new_block_timestamp: Timestamp::default(),
            old_block_timestamp: Timestamp::default(),
            reserve_factor_mantissa: U256::zero(),
            total_borrows: Balance::default(),
            total_reserves: Balance::default(),
        };
        let out = calculate_interest(&input);
        assert_eq!(out.err().unwrap(), Error::BorrowRateIsAbsurdlyHigh)
    }

    #[test]
    fn test_calculate_interest() {
        let old_timestamp = Timestamp::default();
        let inputs: &[CalculateInterestInput] = &[
            CalculateInterestInput {
                old_block_timestamp: old_timestamp,
                new_block_timestamp: old_timestamp + mantissa().as_u64(),
                borrow_index: Exp {
                    mantissa: WrappedU256::from(U256::zero()),
                },
                borrow_rate: mantissa().div(100000), // 0.001 %
                reserve_factor_mantissa: mantissa().div(100), // 1 %
                total_borrows: 10_000 * (10_u128.pow(18)),
                total_reserves: 10_000 * (10_u128.pow(18)),
            },
            CalculateInterestInput {
                old_block_timestamp: old_timestamp,
                new_block_timestamp: old_timestamp + 1000 * 60 * 60, // 1 hour
                borrow_index: Exp {
                    mantissa: WrappedU256::from(U256::from(123123123)),
                },
                borrow_rate: mantissa().div(1000000),
                reserve_factor_mantissa: mantissa().div(10),
                total_borrows: 100_000 * (10_u128.pow(18)),
                total_reserves: 1_000_000 * (10_u128.pow(18)),
            },
            CalculateInterestInput {
                old_block_timestamp: old_timestamp,
                new_block_timestamp: old_timestamp + 999 * 60 * 60 * 2345 * 123,
                borrow_index: Exp {
                    mantissa: WrappedU256::from(U256::from(123123123)),
                },
                borrow_rate: mantissa().div(123123),
                reserve_factor_mantissa: mantissa().div(10).mul(2),
                total_borrows: 123_456 * (10_u128.pow(18)),
                total_reserves: 789_012 * (10_u128.pow(18)),
            },
        ];
        for input in inputs {
            let got = calculate_interest(&input).unwrap();
            let delta = input
                .new_block_timestamp
                .abs_diff(input.old_block_timestamp);
            // interest accumulated should be (borrow rate * delta * total borrows)
            let interest_want = input
                .borrow_rate
                .mul(U256::from(
                    input.new_block_timestamp - input.old_block_timestamp,
                ))
                .mul(U256::from(input.total_borrows))
                .div(mantissa())
                .as_u128();
            let reserves_want = U256::from(input.reserve_factor_mantissa)
                .mul(U256::from(interest_want))
                .div(U256::from(10_u128.pow(18)))
                .add(U256::from(input.total_reserves));
            assert_eq!(got.interest_accumulated, interest_want);
            assert_eq!(got.total_borrows, interest_want + (input.total_borrows));
            assert_eq!(got.total_reserves, reserves_want.as_u128());
            let borrow_idx_want = input
                .borrow_rate
                .mul(U256::from(delta))
                .mul(U256::from(input.borrow_index.mantissa))
                .div(U256::from(10_u128.pow(18)))
                .add(U256::from(input.borrow_index.mantissa));
            assert_eq!(U256::from(got.borrow_index.mantissa), borrow_idx_want);
        }
    }

    #[test]
    // protocol_seize_tokens = seizeTokens * protocolSeizeShare
    // liquidator_seize_tokens = seizeTokens - (seizeTokens * protocolSeizeShare)
    // protocol_seize_amount = exchangeRate * protocolSeizeTokens
    fn test_protocol_seize_amount() {
        // 1%
        let exchange_rate = Exp {
            mantissa: (WrappedU256::from(
                U256::from(10)
                    .pow(U256::from(18))
                    .mul(U256::one())
                    .div(U256::from(100)),
            )),
        };
        let seize_tokens = 10_u128.pow(18).mul(100000000000);
        let protocol_seize_tokens = seize_tokens.mul(10).div(100);
        let protocol_seize_share_mantissa = U256::from(10_u128.pow(18).div(10)); // 10%
        let liquidator_seize_tokens_want = seize_tokens.mul(9).div(10);
        let protocol_seize_amount_want = protocol_seize_tokens.mul(1).div(100); // 1%
        let (liquidator_seize_tokens_got, protocol_seize_amount_got, protocol_seize_tokens_got) =
            protocol_seize_amount(exchange_rate, seize_tokens, protocol_seize_share_mantissa);
        assert_eq!(liquidator_seize_tokens_got, liquidator_seize_tokens_want);
        assert_eq!(protocol_seize_amount_got, protocol_seize_amount_want);
        assert_eq!(protocol_seize_tokens_got, protocol_seize_tokens);
    }
    #[test]
    fn test_exchange_rate_in_case_total_supply_is_zero() {
        assert_eq!(exchange_rate(0, 1, 1, 1), U256::zero());
    }

    #[test]
    fn test_exchange_rate() {
        let with_dec = |val: u128| 10_u128.pow(18).mul(val);
        struct Case {
            total_cash: u128,
            total_borrows: u128,
            total_reserves: u128,
            total_supply: u128,
        }
        let cases: &[Case] = &[
            Case {
                total_cash: with_dec(999987),
                total_borrows: with_dec(199987),
                total_reserves: with_dec(299987),
                total_supply: with_dec(1999987),
            },
            Case {
                total_cash: with_dec(999983),
                total_borrows: with_dec(199983),
                total_reserves: with_dec(299983),
                total_supply: with_dec(1999983),
            },
            Case {
                total_cash: with_dec(1999983),
                total_borrows: with_dec(1199983),
                total_reserves: with_dec(1299983),
                total_supply: with_dec(11999983),
            },
            Case {
                total_cash: with_dec(1234567),
                total_borrows: with_dec(234567),
                total_reserves: with_dec(34567),
                total_supply: with_dec(11999983),
            },
        ];
        for case in cases {
            let rate_want = U256::from(10_u128.pow(18))
                .mul(U256::from(
                    case.total_cash + case.total_borrows - case.total_reserves,
                ))
                .div(U256::from(case.total_supply));
            assert_eq!(
                exchange_rate(
                    case.total_supply,
                    case.total_cash,
                    case.total_borrows,
                    case.total_reserves
                ),
                rate_want
            )
        }
    }
}
