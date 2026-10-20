mod index_range;
pub use index_range::{IndexRange, IndexRangeIterator, SemiboundedIndexRange};

use num_integer::Integer;
use num_traits::PrimInt;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub trait RawIndex:
    Integer + PrimInt + Copy + Sub<Output = Self> + Display + Default + Debug + Ord + Hash
{
    fn as_usize(self) -> usize;
    fn from_usize(val: usize) -> Self;
    fn try_from_usize(val: usize) -> Option<Self>;
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

    fn try_from_usize(val: usize) -> Option<Self> {
        if val <= Self::MAX as usize {
            Some(val as Self)
        } else {
            None
        }
    }
}
impl RawIndex for u16 {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(val: usize) -> Self {
        val as Self
    }

    fn try_from_usize(val: usize) -> Option<Self> {
        if val <= Self::MAX as usize {
            Some(val as Self)
        } else {
            None
        }
    }
}
impl RawIndex for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(val: usize) -> Self {
        val as Self
    }

    fn try_from_usize(val: usize) -> Option<Self> {
        if val <= Self::MAX as usize {
            Some(val as Self)
        } else {
            None
        }
    }
}
impl RawIndex for u64 {
    fn as_usize(self) -> usize {
        self as usize
    }

    fn from_usize(val: usize) -> Self {
        val as Self
    }

    fn try_from_usize(val: usize) -> Option<Self> {
        Some(val as Self)
    }
}
impl RawIndex for usize {
    fn as_usize(self) -> usize {
        self
    }

    fn from_usize(val: usize) -> Self {
        val
    }

    fn try_from_usize(val: usize) -> Option<Self> {
        Some(val)
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
    + SubAssign<Self::RawType>
    + Ord
    + Hash
{
    type RawType: RawIndex;

    fn from_raw(index: Self::RawType) -> Self;
    fn raw(self) -> Self::RawType;

    /// Converts this index into an index of another type. The underlying index value remains the
    /// same.
    ///
    /// This is useful for identity maps, e.g. mapping the state index to the choice index in DTMCs.
    /// In debug builds, this panics if the index does not fit into the raw type of `Target`.
    fn reinterpret_as<Target: Index>(self) -> Target {
        let raw = self.raw().as_usize();
        debug_assert!(
            Target::RawType::try_from_usize(raw).is_some(),
            "index {raw} does not fit into the raw type of the target index"
        );
        Target::from_raw(Target::RawType::from_usize(raw))
    }
}

/// Defines an index type.
///
/// `index!(Name)` creates an index of form `Name<Raw: RawIndex>`, i.e. one that internally uses
/// a type implementing `RawIndex`. `RawIndex` is implemented by `u8`, `u16`, `u32` and `usize`.
///
/// `index!(Name<P: Bound> => RawType)` creates an index of form `Name<P: Bound>` and internally
/// uses `RawType` for storage. `RawType` may depend on `P`. This can be useful for index profiles:
/// You create a trait `Ix` with e.g. types `type State:
///
/// defines `Name<P: Bound>`, backed by `RawType`, which may
/// depend on `P` (e.g. `index!(StateIndex<I: Ix> => I::State)`).
#[macro_export]
macro_rules! index {
    ($(#[$attr:meta])* $name:ident < $param:ident : $bound:path > => $raw:ty) => {
        $(#[$attr])*
        #[derive(Copy, Clone, PartialEq, Eq, Default, Ord, PartialOrd, Hash)]
        pub struct $name<$param: $bound> {
            raw: $raw,
        }

        impl<$param: $bound> $crate::Index for $name<$param> {
            type RawType = $raw;

            fn from_raw(index: Self::RawType) -> Self {
                Self { raw: index }
            }

            fn raw(self) -> Self::RawType {
                self.raw
            }
        }

        impl<$param: $bound> std::fmt::Debug for $name<$param> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($name({})), self.raw)
            }
        }

        impl<$param: $bound> std::ops::Add<$raw> for $name<$param> {
            type Output = Self;

            fn add(self, rhs: $raw) -> Self {
                use $crate::Index;
                Self::from_raw(self.raw + rhs)
            }
        }

        impl<$param: $bound> std::ops::AddAssign<$raw> for $name<$param> {
            fn add_assign(&mut self, rhs: $raw) {
                self.raw = self.raw + rhs
            }
        }

        impl<$param: $bound> std::ops::Sub<$raw> for $name<$param> {
            type Output = Self;

            fn sub(self, rhs: $raw) -> Self::Output {
                use $crate::Index;
                Self::from_raw(self.raw - rhs)
            }
        }

        impl<$param: $bound> std::ops::SubAssign<$raw> for $name<$param> {
            fn sub_assign(&mut self, rhs: $raw) {
                self.raw = self.raw - rhs
            }
        }

        impl<$param: $bound> std::ops::Mul<$raw> for $name<$param> {
            type Output = Self;

            fn mul(self, rhs: $raw) -> Self::Output {
                use $crate::Index;
                Self::from_raw(self.raw * rhs)
            }
        }

        impl<$param: $bound> std::ops::Div<$raw> for $name<$param> {
            type Output = Self;

            fn div(self, rhs: $raw) -> Self::Output {
                use $crate::Index;
                Self::from_raw(self.raw / rhs)
            }
        }
    };
    ($(#[$attr:meta])* $name:ident) => {
        $crate::index!($(#[$attr])* $name<Raw: $crate::RawIndex> => Raw);
    };
}
