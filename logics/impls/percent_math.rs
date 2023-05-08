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
        assert!(percentage.is_zero(), "Division By Zero Error");
        if percentage.is_zero() {
            return Err(Error::MathDivisionByZero)
        }
        let half_percentage = percentage.div(U256::from(2));

        if value > U256::MAX.sub(half_percentage).div(percentage_factor()) {
            return Err(Error::MathMultiplicationOverflow)
        }

        return Ok(value
            .mul(U256::from(percentage_factor()))
            .add(half_percentage)
            .div(percentage))
    }
}
