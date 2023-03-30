#![allow(unused)]
use core::{
    ops::{
        Add,
        Div,
        Mul,
        Sub,
    },
    str::FromStr,
};

use openbrush::traits::String;
use primitive_types::U256;

use crate::traits::types::WrappedU256;
pub fn exp_scale() -> U256 {
    U256::from(10_u128.pow(18))
}
pub fn ray_scale() -> U256 {
    U256::from(10_u128.pow(27))
}
fn wad_ray_ratio() -> U256 {
    U256::from(10_u128.pow(9))
}
fn half_exp_scale() -> U256 {
    exp_scale().div(2)
}
fn mantissa_one() -> U256 {
    exp_scale()
}

#[derive(Clone, Debug)]
pub struct Exp {
    pub mantissa: WrappedU256,
}
pub struct Ray {
    pub mantissa: WrappedU256,
}

pub fn fraction(one: WrappedU256, another: WrappedU256) -> Ray {
    Ray {
        mantissa: WrappedU256::from(U256::from(one).mul(ray_scale()).div(U256::from(another))),
    }
}

impl Exp {
    fn add(&self, a: Exp) -> Exp {
        self._op(a, |o, v| o.add(v))
    }
    fn sub(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.sub(v))
    }
    pub fn to_ray(&self) -> Ray {
        Ray {
            mantissa: WrappedU256::from(U256::from(self.mantissa).mul(wad_ray_ratio())),
        }
    }
    pub fn mul(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.mul(v).div(exp_scale()))
    }

    pub fn mul_scalar(&self, mantissa: U256) -> Exp {
        Exp {
            mantissa: WrappedU256::from(U256::from(self.mantissa).mul(mantissa)),
        }
    }

    pub fn div(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.mul(exp_scale()).div(v))
    }
    pub fn mul_scalar_truncate(&self, scalar: U256) -> U256 {
        let product = self.mul_scalar(scalar);
        product._trunc()
    }
    pub fn mul_scalar_truncate_add_uint(&self, scalar: U256, addend: U256) -> U256 {
        self.mul_scalar_truncate(scalar).add(addend)
    }

    fn lt(&self, another: Exp) -> bool {
        self._cmp(another, |a, b| a.lt(&b))
    }

    fn le(&self, another: Exp) -> bool {
        self._cmp(another, |a, b| a.le(&b))
    }

    fn gt(&self, another: Exp) -> bool {
        self._cmp(another, |a, b| a.gt(&b))
    }
    fn ge(&self, another: Exp) -> bool {
        self._cmp(another, |a, b| a.ge(&b))
    }

    fn is_zero(&self) -> bool {
        U256::from(self.mantissa).is_zero()
    }

    fn _cmp(&self, another: Exp, comparator: fn(left: U256, right: U256) -> bool) -> bool {
        comparator(U256::from(self.mantissa), U256::from(another.mantissa))
    }

    fn _op(&self, a: Exp, op: fn(one: U256, another: U256) -> U256) -> Exp {
        Exp {
            mantissa: WrappedU256::from(op(U256::from(self.mantissa), U256::from(a.mantissa))),
        }
    }
    pub fn truncate(&self) -> U256 {
        self._trunc()
    }
    fn _trunc(&self) -> U256 {
        U256::from(self.mantissa).div(exp_scale())
    }
}

impl Ray {
    pub fn add(&self, a: Ray) -> Ray {
        self._op(a, |o, v| o.add(v))
    }
    pub fn sub(&self, another: Ray) -> Ray {
        self._op(another, |o, v| o.sub(v))
    }

    pub fn to_exp(&self) -> Exp {
        let half_ratio = wad_ray_ratio().div(2);
        Exp {
            mantissa: WrappedU256::from(
                half_ratio
                    .add(U256::from(self.mantissa))
                    .div(wad_ray_ratio()),
            ),
        }
    }

    pub fn mul(&self, another: Ray) -> Ray {
        self._op(another, |o, v| o.mul(v).div(ray_scale()))
    }
    fn div(&self, another: Ray) -> Ray {
        self._op(another, |o, v| o.mul(ray_scale()).div(v))
    }
    fn _op(&self, a: Ray, op: fn(one: U256, another: U256) -> U256) -> Ray {
        Ray {
            mantissa: WrappedU256::from(op(U256::from(self.mantissa), U256::from(a.mantissa))),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use primitive_types::U256;
    fn wr(val: u128) -> WrappedU256 {
        WrappedU256::from(U256::from(val))
    }
    #[test]
    fn test_add() {
        let a = Exp { mantissa: wr(1) };
        let b = Exp { mantissa: wr(1) };
        assert_eq!(U256::from(2), a.add(b).mantissa.into())
    }
    #[test]
    fn test_sub() {
        let a = Exp { mantissa: wr(2) };
        let b = Exp { mantissa: wr(1) };
        assert_eq!(U256::one(), a.sub(b).mantissa.into())
    }
    #[test]
    fn test_mul() {
        let a = Exp { mantissa: wr(2) };
        let b = Exp {
            mantissa: WrappedU256::from(U256::from(2).mul(exp_scale())),
        };
        assert_eq!(U256::from(4), a.mul(b).mantissa.into())
    }
    #[test]
    fn test_mul_scalar() {
        let a = Exp { mantissa: wr(2) };
        let b = U256::from(2);
        assert_eq!(U256::from(4), a.mul_scalar(b).mantissa.into())
    }
    #[test]
    fn test_div() {
        let out: i128 = 1666666666666666666;
        let a = Exp { mantissa: wr(5) };
        let b = Exp { mantissa: wr(3) };
        assert_eq!(U256::from(out), a.div(b).mantissa.into())
    }
    #[test]
    fn test_mul_scalar_truncate() {
        let a = Exp {
            mantissa: WrappedU256::from(U256::from(10).mul(exp_scale())),
        };
        let b = U256::from(5);
        assert_eq!(U256::from(50), a.mul_scalar_truncate(b))
    }
    #[test]
    fn test_mul_scalar_truncate_add_uint() {
        let a = Exp {
            mantissa: WrappedU256::from(U256::from(10).mul(exp_scale())),
        };
        let b = U256::from(5);
        let c = U256::from(10);

        assert_eq!(U256::from(60), a.mul_scalar_truncate_add_uint(b, c))
    }
    #[test]
    fn test_truncate() {
        let val: i128 = 1_111_111_111_111_111_111;
        let a = Exp {
            mantissa: WrappedU256::from(U256::from(val)),
        };
        assert_eq!(U256::one(), a.truncate())
    }
}
