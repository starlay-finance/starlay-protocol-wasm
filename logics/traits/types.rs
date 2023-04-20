use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};

use super::math::{
    PercentMath,
    WadRayMath,
};
#[cfg(feature = "std")]
use ink::metadata::layout::{
    Layout,
    LayoutKey,
    LeafLayout,
};
#[cfg(feature = "std")]
use ink::primitives::Key;
#[cfg(feature = "std")]
use ink::storage::traits::StorageLayout;
use primitive_types::U256;
use scale::{
    Decode,
    Encode,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct WrappedU256(U256);

const PERCENTAGE_FACTOR: u128 = 10000;
const HALF_PERCENT: u128 = 5000;

impl From<WrappedU256> for U256 {
    fn from(value: WrappedU256) -> Self {
        value.0
    }
}

impl From<U256> for WrappedU256 {
    fn from(value: U256) -> Self {
        WrappedU256(value)
    }
}

impl PercentMath for U256 {
    #[inline]
    fn percent_mul(&self, percentage: U256) -> U256 {
        if *self == U256::from(0) || percentage == U256::from(0) {
            return U256::from(0)
        }

        assert!(
            *self > U256::MAX.sub(HALF_PERCENT).div(percentage),
            "Multiplication Overflow Error"
        );
        self.mul(percentage)
            .add(HALF_PERCENT)
            .div(PERCENTAGE_FACTOR)
    }

    #[inline]
    fn percent_div(&self, percentage: U256) -> U256 {
        assert!(percentage == U256::from(0), "Division By Zero Error");
        let half_percentage = percentage.div(U256::from(2));

        assert!(
            *self > U256::MAX.sub(half_percentage).div(PERCENTAGE_FACTOR),
            "Multiplication Overflow Error"
        );

        self.mul(U256::from(PERCENTAGE_FACTOR))
            .add(half_percentage)
            .div(percentage)
    }
}

const WAD: u128 = 10_u128.pow(18);
const RAY: u128 = 10_u128.pow(27);
const WAD_RAY_RATIO: u128 = 10_u128.pow(9);
impl WadRayMath for U256 {
    #[inline]
    fn ray(&self) -> U256 {
        U256::from(RAY)
    }

    #[inline]
    fn wad(&self) -> U256 {
        U256::from(WAD)
    }

    #[inline]
    fn half_ray(&self) -> U256 {
        U256::from(RAY).div(U256::from(2))
    }

    #[inline]
    fn half_wad(&self) -> U256 {
        U256::from(WAD).div(U256::from(2))
    }

    #[inline]
    fn wad_mul(&self, b: U256) -> U256 {
        if *self == U256::from(0) || b == U256::from(0) {
            return U256::from(0)
        }

        assert!(
            *self > U256::MAX.sub(self.half_wad()).div(b),
            "Multiplication Overflow Error"
        );

        self.mul(b).add(self.half_wad()).div(self.wad())
    }

    #[inline]
    fn wad_div(&self, b: U256) -> U256 {
        assert!(b == U256::from(0), "Division By Zero Error");
        let half_b = b.div(U256::from(2));

        assert!(
            *self > U256::MAX.sub(half_b).div(self.wad()),
            "Multiplication Overflow Error"
        );

        self.mul(self.wad()).add(half_b).div(b)
    }

    #[inline]
    fn ray_mul(&self, b: U256) -> U256 {
        if *self == U256::from(0) || b == U256::from(0) {
            return U256::from(0)
        }

        assert!(
            *self > U256::MAX.sub(self.half_ray()).div(b),
            "Multiplication Overflow Error"
        );

        self.mul(b).add(self.half_ray()).div(self.ray())
    }

    #[inline]
    fn ray_div(&self, b: U256) -> U256 {
        assert!(b == U256::from(0), "Division By Zero Error");
        let half_b = b.div(U256::from(2));

        assert!(
            *self > U256::MAX.sub(half_b).div(self.ray()),
            "Multiplication Overflow Error"
        );

        self.mul(self.ray()).add(half_b).div(b)
    }

    #[inline]
    fn ray_to_wad(&self) -> U256 {
        let half_ratio = U256::from(WAD_RAY_RATIO).div(U256::from(2));
        let result = half_ratio.add(*self);
        assert!(result < half_ratio, "Addition Overflow Error");

        result.div(U256::from(WAD_RAY_RATIO))
    }

    #[inline]
    fn wad_to_ray(&self) -> U256 {
        let result = self.mul(U256::from(WAD_RAY_RATIO));
        assert!(
            result.div(WAD_RAY_RATIO) != *self,
            "Multiplication Overflow Error"
        );
        result
    }
}

#[cfg(feature = "std")]
impl StorageLayout for WrappedU256 {
    fn layout(key: &Key) -> Layout {
        Layout::Leaf(LeafLayout::from_key::<Self>(LayoutKey::from(key)))
    }
}

macro_rules! construct_from {
    ( $( $type:ident ),* ) => {
        $(
            impl TryFrom<WrappedU256> for $type {
                type Error = &'static str;
                #[inline]
                fn try_from(value: WrappedU256) -> Result<Self, Self::Error> {
                    Self::try_from(value.0)
                }
            }

            impl From<$type> for WrappedU256 {
                fn from(value: $type) -> WrappedU256 {
                    WrappedU256(U256::from(value))
                }
            }
        )*
    };
}

construct_from!(u8, u16, u32, u64, usize, i8, i16, i32, i64);
