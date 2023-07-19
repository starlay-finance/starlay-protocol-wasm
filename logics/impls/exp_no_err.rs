// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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

use super::wad_ray_math::{
    exp_ray_ratio,
    Ray,
};
pub fn exp_scale() -> U256 {
    U256::from(10_u128.pow(18))
}

pub fn half_exp_scale() -> U256 {
    exp_scale().div(2)
}
fn mantissa_one() -> U256 {
    exp_scale()
}

#[derive(Clone, Debug)]
pub struct Exp {
    pub mantissa: WrappedU256,
}

impl Exp {
    pub fn add(&self, a: Exp) -> Exp {
        self._op(a, |o, v| o.add(v))
    }

    pub fn sub(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.sub(v))
    }
    pub fn to_ray(&self) -> Ray {
        Ray {
            mantissa: WrappedU256::from(U256::from(self.mantissa).mul(exp_ray_ratio())),
        }
    }
    pub fn mul(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.mul(v).div(exp_scale()))
    }

    pub fn mul_scalar(&self, scalar: U256) -> Exp {
        Exp {
            mantissa: WrappedU256::from(U256::from(self.mantissa).mul(scalar)),
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
