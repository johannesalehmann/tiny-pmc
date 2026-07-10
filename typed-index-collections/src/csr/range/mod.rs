mod iterator;
pub use iterator::CsrRangeIterator;

use crate::Index;
use crate::index::RawIndex;
use num_integer::Integer;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsrRange<To: Index> {
    pub(super) start: To::RawType,
    pub(super) end: To::RawType,
    pub(super) phantom_data: PhantomData<To>,
}

impl<To: Index> CsrRange<To> {
    pub fn len(&self) -> To::RawType {
        self.end - self.start
    }

    pub fn identity<From: Index>(index: From) -> Self {
        let start = index.raw().as_usize();
        Self {
            start: To::RawType::from_usize(start),
            end: To::RawType::from_usize(start + 1),
            phantom_data: PhantomData,
        }
    }

    pub fn start(&self) -> To {
        To::from_raw(self.start)
    }

    pub fn end(&self) -> To {
        To::from_raw(self.end)
    }
}

impl<To: Index> IntoIterator for CsrRange<To> {
    type Item = To;
    type IntoIter = CsrRangeIterator<To>;

    fn into_iter(self) -> Self::IntoIter {
        CsrRangeIterator {
            from: self.start,
            to: self.end,
            next_entry: self.start,
        }
    }
}
