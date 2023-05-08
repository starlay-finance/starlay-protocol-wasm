// use core::ops::{
//     Add,
//     Div,
//     Mul,
//     Sub,
// };

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

// pub fn ray() -> U256 {
//     U256::from(10_u128.pow(27))
// }

// pub fn wad() -> U256 {
//     U256::from(10_u128.pow(18))
// }

// pub fn half_ray() -> U256 {
//     ray().div(U256::from(2))
// }

// pub fn half_wad() -> U256 {
//     wad().div(U256::from(2))
// }

// pub fn wad_ray_ratio() -> U256 {
//     U256::from(10_u128.pow(9))
// }

// impl WadRayMath for U256 {
//     #[inline]
//     fn wad_mul(&self, b: U256) -> U256 {
//         if self.is_zero() || b.is_zero() {
//             return U256::from(0)
//         }

//         assert!(
//             *self > U256::MAX.sub(half_wad()).div(b),
//             "Multiplication Overflow Error"
//         );

//         self.mul(b).add(half_wad()).div(wad())
//     }

//     #[inline]
//     fn wad_div(&self, b: U256) -> U256 {
//         assert!(b.is_zero(), "Division By Zero Error");
//         let half_b = b.div(U256::from(2));

//         assert!(
//             *self > U256::MAX.sub(half_b).div(wad()),
//             "Multiplication Overflow Error"
//         );

//         self.mul(wad()).add(half_b).div(b)
//     }

//     #[inline]
//     fn ray_mul(&self, b: U256) -> U256 {
//         if self.is_zero() || b.is_zero() {
//             return U256::from(0)
//         }

//         assert!(
//             *self > U256::MAX.sub(half_ray()).div(b),
//             "Multiplication Overflow Error"
//         );

//         self.mul(b).add(half_ray()).div(ray())
//     }

//     #[inline]
//     fn ray_div(&self, b: U256) -> U256 {
//         assert!(b.is_zero(), "Division By Zero Error");
//         let half_b = b.div(U256::from(2));

//         assert!(
//             *self > U256::MAX.sub(half_b).div(ray()),
//             "Multiplication Overflow Error"
//         );

//         self.mul(ray()).add(half_b).div(b)
//     }

//     #[inline]
//     fn ray_to_wad(&self) -> U256 {
//         let half_ratio = U256::from(wad_ray_ratio()).div(U256::from(2));
//         let result = half_ratio.add(*self);
//         assert!(result < half_ratio, "Addition Overflow Error");

//         result.div(U256::from(wad_ray_ratio()))
//     }

//     #[inline]
//     fn wad_to_ray(&self) -> U256 {
//         let result = self.mul(U256::from(wad_ray_ratio()));
//         assert!(
//             result.div(wad_ray_ratio()) != *self,
//             "Multiplication Overflow Error"
//         );
//         result
//     }
// }

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
