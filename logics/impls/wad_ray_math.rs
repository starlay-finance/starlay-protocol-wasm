use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};
#[derive(Debug)]
pub enum Error {
    MathMultiplicationOverflow,
    MathDivisionByZero,
}
use primitive_types::U256;

use crate::traits::types::WrappedU256;

use super::exp_no_err::{
    exp_scale,
    half_exp_scale,
    Exp,
};

pub fn ray_scale() -> U256 {
    U256::from(10_u128.pow(27))
}

fn half_ray() -> U256 {
    ray_scale().div(2)
}
pub fn exp_ray_ratio() -> U256 {
    U256::from(10_u128.pow(9))
}
pub struct Ray {
    pub mantissa: WrappedU256,
}
type Wad = Exp;
impl Wad {
    pub fn wad_mul(&self, another: Wad) -> Result<Wad, Error> {
        let a = U256::from(self.mantissa);
        let b = U256::from(another.mantissa);
        if (a.eq(&U256::zero()) || b.eq(&U256::zero())) {
            return Ok(Wad {
                mantissa: WrappedU256::from(U256::from(0)),
            })
        }
        if (a.gt(&U256::max_value().sub(half_exp_scale().div(b)))) {
            return Err(Error::MathMultiplicationOverflow)
        }
        Ok(Wad {
            mantissa: WrappedU256::from(a.mul(b).add(half_exp_scale()).div(exp_scale())),
        })
    }
    pub fn wad_div(&self, another: Wad) -> Result<Wad, Error> {
        let a = U256::from(self.mantissa);
        let b = U256::from(another.mantissa);
        if U256::from(b).is_zero() {
            return Err(Error::MathDivisionByZero)
        }
        let half_another = b.div(U256::from(2));
        if (U256::from(a).gt(&U256::max_value().sub(half_another).div(exp_scale()))) {
            return Err(Error::MathMultiplicationOverflow)
        }
        Ok(Wad {
            mantissa: U256::from(a)
                .mul(exp_scale())
                .add(half_another)
                .div(b)
                .into(),
        })
    }
}

pub fn fraction(one: WrappedU256, another: WrappedU256) -> Ray {
    Ray {
        mantissa: WrappedU256::from(U256::from(one).mul(ray_scale()).div(U256::from(another))),
    }
}

impl Ray {
    pub fn add(&self, a: Ray) -> Ray {
        self._op(a, |o, v| o.add(v))
    }
    pub fn sub(&self, another: Ray) -> Ray {
        self._op(another, |o, v| o.sub(v))
    }
    fn _op(&self, a: Ray, op: fn(one: U256, another: U256) -> U256) -> Ray {
        Ray {
            mantissa: WrappedU256::from(op(U256::from(self.mantissa), U256::from(a.mantissa))),
        }
    }

    pub fn to_exp(&self) -> Exp {
        let half_ratio = exp_ray_ratio().div(2);
        Exp {
            mantissa: WrappedU256::from(
                half_ratio
                    .add(U256::from(self.mantissa))
                    .div(exp_ray_ratio()),
            ),
        }
    }

    pub fn ray_mul(&self, another: Ray) -> Result<Ray, Error> {
        if U256::from(self.mantissa).is_zero() || U256::from(another.mantissa).is_zero() {
            return Ok(Ray {
                mantissa: WrappedU256::from(U256::from(0)),
            })
        }
        if (U256::from(self.mantissa).gt(&U256::max_value().sub(half_ray()).div(another.mantissa)))
        {
            return Err(Error::MathMultiplicationOverflow)
        }
        Ok(Ray {
            mantissa: U256::from(self.mantissa)
                .mul(U256::from(another.mantissa))
                .add(half_ray())
                .div(ray_scale())
                .into(),
        })
    }
    pub fn ray_div(&self, another: Ray) -> Result<Ray, Error> {
        if U256::from(another.mantissa).is_zero() {
            return Err(Error::MathDivisionByZero)
        }
        let half_another = U256::from(another.mantissa).div(U256::from(2));
        if (U256::from(self.mantissa).gt(&U256::max_value().sub(half_another).div(ray_scale()))) {
            return Err(Error::MathMultiplicationOverflow)
        }
        Ok(Ray {
            mantissa: U256::from(self.mantissa)
                .mul(ray_scale())
                .add(half_another)
                .div(U256::from(another.mantissa))
                .into(),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use primitive_types::U256;
    fn wr(val: u128) -> WrappedU256 {
        WrappedU256::from(U256::from(val))
    }
}
