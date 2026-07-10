use num_integer::Integer;
use num_traits::PrimInt;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul, Sub};

pub trait RawIndex: Integer + PrimInt + Copy + Sub<Output = Self> + Display + Default {
    fn as_usize(self) -> usize;
    fn from_usize(val: usize) -> Self;
    fn zero() -> Self {
        <Self as num_traits::Zero>::zero()
    }
    fn one() -> Self {
        <Self as num_traits::One>::one()
    }
}
impl RawIndex for u8 {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(val: usize) -> Self {
        val as Self
    }
}
impl RawIndex for u16 {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(val: usize) -> Self {
        val as Self
    }
}
impl RawIndex for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(val: usize) -> Self {
        val as Self
    }
}
impl RawIndex for u64 {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(val: usize) -> Self {
        val as Self
    }
}
impl RawIndex for usize {
    fn as_usize(self) -> usize {
        self
    }

    fn from_usize(val: usize) -> Self {
        val
    }
}

pub trait Index:
    Copy
    + Eq
    + Debug
    + Default
    + Add<Self::RawType, Output = Self>
    + Sub<Self::RawType, Output = Self>
    + Mul<Self::RawType, Output = Self>
    + Div<Self::RawType, Output = Self>
    + AddAssign<Self::RawType>
{
    type RawType: RawIndex;

    fn from_raw(index: Self::RawType) -> Self;
    fn raw(self) -> Self::RawType;
}

#[macro_export]
macro_rules! index {
    ($name: ident) => {
        #[derive(Copy, Clone, PartialEq, Eq, Default)]
        pub struct $name<Raw: typed_index_collections::RawIndex> {
            raw: Raw,
        }

        impl<Raw: typed_index_collections::RawIndex> typed_index_collections::Index for $name<Raw> {
            type RawType = Raw;

            fn from_raw(index: Self::RawType) -> Self {
                Self { raw: index }
            }

            fn raw(self) -> Self::RawType {
                self.raw
            }
        }

        impl<Raw: typed_index_collections::RawIndex> std::fmt::Debug for $name<Raw> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($name({})), self.raw)
            }
        }

        impl<Raw: typed_index_collections::RawIndex> std::ops::Add<Raw> for $name<Raw> {
            type Output = Self;

            fn add(self, rhs: Raw) -> Self {
                use typed_index_collections::Index;
                Self::from_raw(self.raw + rhs)
            }
        }

        impl<Raw: typed_index_collections::RawIndex> std::ops::AddAssign<Raw> for $name<Raw> {
            fn add_assign(&mut self, rhs: Raw) {
                self.raw = self.raw + rhs
            }
        }

        impl<Raw: typed_index_collections::RawIndex> std::ops::Sub<Raw> for $name<Raw> {
            type Output = Self;

            fn sub(self, rhs: Raw) -> Self::Output {
                use typed_index_collections::Index;
                Self::from_raw(self.raw - rhs)
            }
        }

        impl<Raw: typed_index_collections::RawIndex> std::ops::Mul<Raw> for $name<Raw> {
            type Output = Self;

            fn mul(self, rhs: Raw) -> Self::Output {
                use typed_index_collections::Index;
                Self::from_raw(self.raw * rhs)
            }
        }

        impl<Raw: typed_index_collections::RawIndex> std::ops::Div<Raw> for $name<Raw> {
            type Output = Self;

            fn div(self, rhs: Raw) -> Self::Output {
                use typed_index_collections::Index;
                Self::from_raw(self.raw / rhs)
            }
        }
    };
}
