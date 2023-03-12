#![allow(unused)]
// TODO: test
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
fn exp_scale() -> U256 {
    U256::from_dec_str("10")
        .unwrap()
        .pow(U256::from_str("18").unwrap())
}
fn double_scale() -> U256 {
    U256::from_dec_str("10")
        .unwrap()
        .pow(U256::from_str("36").unwrap())
}
fn half_exp_scale() -> U256 {
    exp_scale().div(2)
}
fn mantissa_one() -> U256 {
    exp_scale()
}

pub struct Exp {
    mantissa: WrappedU256,
}
pub struct Double {
    mantissa: WrappedU256,
}

pub fn fraction(one: WrappedU256, another: WrappedU256) -> Double {
    Double {
        mantissa: WrappedU256::from(U256::from(one).mul(double_scale()).div(U256::from(another))),
    }
}

impl Exp {
    fn add(&self, a: Exp) -> Exp {
        self._op(a, |o, v| o.add(v))
    }
    fn sub(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.sub(v))
    }

    fn mul(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.mul(v).div(exp_scale()))
    }

    fn div(&self, another: Exp) -> Exp {
        self._op(another, |o, v| o.mul(exp_scale()).div(v))
    }
    fn mul_scalar_truncate(&self, scalar: u128) -> U256 {
        let product = self.mul(Exp {
            mantissa: WrappedU256::from(U256::from(scalar)),
        });
        product._trunc()
    }
    fn mul_scalar_truncate_add_uint(&self, scalar: u128, addend: WrappedU256) -> U256 {
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
    fn _trunc(&self) -> U256 {
        U256::from(self.mantissa).div(exp_scale())
    }
}

impl Double {
    fn add(&self, a: Double) -> Double {
        self._op(a, |o, v| o.add(v))
    }
    fn sub(&self, another: Double) -> Double {
        self._op(another, |o, v| o.sub(v))
    }

    fn mul(&self, another: Double) -> Double {
        self._op(another, |o, v| o.mul(v).div(double_scale()))
    }
    fn div(&self, another: Double) -> Double {
        self._op(another, |o, v| o.mul(double_scale()).div(v))
    }
    fn _op(&self, a: Double, op: fn(one: U256, another: U256) -> U256) -> Double {
        Double {
            mantissa: WrappedU256::from(op(U256::from(self.mantissa), U256::from(a.mantissa))),
        }
    }
}
