use num_integer::Integer;
use num_traits::{MulAdd, PrimInt};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Sub};

pub trait RawIndex: Integer + PrimInt + Copy + Sub<Output = Self> + Display + Default {
    fn as_usize(self) -> usize;
    fn from_usize(val: usize) -> Self;
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

pub trait Index: Copy + Eq + Debug {
    type RawType: RawIndex;

    fn from_raw(index: Self::RawType) -> Self;
    fn raw(self) -> Self::RawType;
}

macro_rules! index {
    ($name: ident) => {
        #[derive(Copy, Clone, PartialEq, Eq, Default)]
        pub struct $name<Raw: RawIndex> {
            raw: Raw,
        }

        impl<Raw: RawIndex> Index for $name<Raw> {
            type RawType = Raw;

            fn from_raw(index: Self::RawType) -> Self {
                Self { raw: index }
            }

            fn raw(self) -> Self::RawType {
                self.raw
            }
        }

        impl<Raw: RawIndex> Debug for $name<Raw> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($name({})), self.raw)
            }
        }

        impl<Raw: RawIndex> Add<Raw> for $name<Raw> {
            type Output = Self;

            fn add(self, rhs: Raw) -> Self {
                Self::from_raw(self.raw + rhs)
            }
        }

        impl<Raw: RawIndex> AddAssign<Raw> for $name<Raw> {
            fn add_assign(&mut self, rhs: Raw) {
                self.raw = self.raw + rhs
            }
        }

        impl<Raw: RawIndex> Sub<Raw> for $name<Raw> {
            type Output = Self;

            fn sub(self, rhs: Raw) -> Self::Output {
                Self::from_raw(self.raw - rhs)
            }
        }

        impl<Raw: RawIndex> Mul<Raw> for $name<Raw> {
            type Output = Self;

            fn mul(self, rhs: Raw) -> Self::Output {
                Self::from_raw(self.raw * rhs)
            }
        }

        impl<Raw: RawIndex> Div<Raw> for $name<Raw> {
            type Output = Self;

            fn div(self, rhs: Raw) -> Self::Output {
                Self::from_raw(self.raw / rhs)
            }
        }
    };
}

index!(StateIndex);
index!(ChoiceIndex);
index!(BranchIndex);
index!(PlayerIndex);
index!(AnnotationIndex);
index!(AnnotationEntryIndex);
index!(ValuationClassIndex);
index!(ValuationClassEntryIndex);
index!(ValuationIndex);
