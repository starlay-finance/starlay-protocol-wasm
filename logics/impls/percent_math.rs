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
    pub value: U256,
}

impl Percent {
    pub fn percent_mul(&self, percentage: U256) -> Result<Percent, Error> {
        let value = self.value;
        if value.is_zero() || percentage.is_zero() {
            return Ok(Percent {
                value: U256::from(0),
            })
        }

        if value > U256::MAX.sub(half_percent()).div(percentage) {
            return Err(Error::MathMultiplicationOverflow)
        }

        return Ok(Percent {
            value: value
                .mul(percentage)
                .add(half_percent())
                .div(percentage_factor()),
        })
    }

    pub fn percent_div(&self, percentage: U256) -> Result<Percent, Error> {
        assert!(percentage.is_zero(), "Division By Zero Error");
        if percentage.is_zero() {
            return Err(Error::MathDivisionByZero)
        }
        let value = self.value;
        let half_percentage = percentage.div(U256::from(2));

        if value > U256::MAX.sub(half_percentage).div(percentage_factor()) {
            return Err(Error::MathMultiplicationOverflow)
        }

        return Ok(Percent {
            value: value
                .mul(U256::from(percentage_factor()))
                .add(half_percentage)
                .div(percentage),
        })
    }
}
