use super::wad_ray_math::Error;
use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};
use primitive_types::U256;

fn percentage_factor() -> U256 {
    U256::from(10000)
}

fn half_percent() -> U256 {
    U256::from(5000)
}

pub struct Percent {
    pub percentage: U256,
}

impl Percent {
    pub fn percent_mul(&self, value: U256) -> Result<U256, Error> {
        let percentage = self.percentage;
        if value.is_zero() || percentage.is_zero() {
            return Ok(U256::from(0))
        }

        if value > U256::MAX.sub(half_percent()).div(percentage) {
            return Err(Error::MathMultiplicationOverflow)
        }

        return Ok(value
            .mul(percentage)
            .add(half_percent())
            .div(percentage_factor()))
    }

    pub fn percent_div(&self, value: U256) -> Result<U256, Error> {
        let percentage = self.percentage;
        if percentage.is_zero() {
            return Err(Error::MathDivisionByZero)
        }
        let half_percentage = percentage.div(U256::from(2));

        if value > U256::MAX.sub(half_percentage).div(percentage_factor()) {
            return Err(Error::MathMultiplicationOverflow)
        }

        return Ok(value
            .mul(percentage_factor())
            .add(half_percentage)
            .div(percentage))
    }
}

#[cfg(test)]
mod tests {
    use super::Percent;
    use crate::impls::wad_ray_math::Error;
    use primitive_types::U256;
    #[test]
    fn test_percent_mul_works() {
        struct Case {
            input: Input,
            expected: U256,
        }
        struct Input {
            percentage: U256,
            value: U256,
        }
        let cases = vec![
            Case {
                input: Input {
                    percentage: U256::from(0),
                    value: U256::from(10000),
                },
                expected: U256::from(0),
            },
            Case {
                input: Input {
                    percentage: U256::from(8000),
                    value: U256::from(0),
                },
                expected: U256::from(0),
            },
            Case {
                input: Input {
                    percentage: U256::from(8000),
                    value: U256::from(100000),
                },
                expected: U256::from(80000),
            },
            Case {
                input: Input {
                    percentage: U256::from(5000),
                    value: U256::from(100000),
                },
                expected: U256::from(50000),
            },
        ];
        for case in cases {
            let percent = Percent {
                percentage: case.input.percentage,
            };
            let got = percent.percent_mul(case.input.value);
            assert_eq!(got.is_err(), false);
            assert_eq!(got.unwrap(), case.expected);
        }
    }

    #[test]
    fn test_percent_div_fails() {
        let percent = Percent {
            percentage: U256::from(0),
        };
        let result = percent.percent_div(U256::from(10000));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MathDivisionByZero);
    }

    #[test]
    fn test_percent_div_works() {
        struct Case {
            input: Input,
            expected: U256,
        }
        struct Input {
            percentage: U256,
            value: U256,
        }
        let cases = vec![
            Case {
                input: Input {
                    percentage: U256::from(8000),
                    value: U256::from(0),
                },
                expected: U256::from(0),
            },
            Case {
                input: Input {
                    percentage: U256::from(8000),
                    value: U256::from(100000),
                },
                expected: U256::from(125000),
            },
        ];
        for case in cases {
            let percent = Percent {
                percentage: case.input.percentage,
            };
            let got = percent.percent_div(case.input.value);
            assert_eq!(got.is_err(), false);
            assert_eq!(got.unwrap(), case.expected);
        }
    }
}
