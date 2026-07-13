mod iterator;
pub use iterator::CsrRangeIterator;

use crate::index::RawIndex;
use crate::{Csr, Index, IndexRange};

#[derive(Copy, Clone)]
pub struct CsrRanges<'a, From: Index, To: Index> {
    csr: &'a Csr<From, To>,
}

impl<'a, From: Index, To: Index> CsrRanges<'a, From, To> {
    pub fn new(csr: &'a Csr<From, To>) -> Self {
        Self { csr }
    }
}

impl<'a, From: Index, To: Index> IntoIterator for CsrRanges<'a, From, To> {
    type Item = IndexRange<To>;
    type IntoIter = CsrRangesIterator<'a, From, To>;

    fn into_iter(self) -> Self::IntoIter {
        CsrRangesIterator {
            csr: self.csr,
            next_index: Some(0),
        }
    }
}

pub struct CsrRangesIterator<'a, From: Index, To: Index> {
    csr: &'a Csr<From, To>,
    next_index: Option<usize>,
}

impl<'a, From: Index, To: Index> CsrRangesIterator<'a, From, To> {
    pub fn enumerate(self) -> EnumeratingCsrRangesIterator<'a, From, To> {
        EnumeratingCsrRangesIterator { iterator: self }
    }
}

impl<'a, From: Index, To: Index> Iterator for CsrRangesIterator<'a, From, To> {
    type Item = IndexRange<To>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_index {
            Some(index) => {
                let res = self
                    .csr
                    .get(From::from_raw(From::RawType::from_usize(index)));
                if index + 1 < self.csr.entries.len() {
                    self.next_index = Some(index + 1);
                } else {
                    self.next_index = None;
                }
                res
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.csr.entries.len() - 1;
        (length, Some(length))
    }
}

impl<From: Index, To: Index> ExactSizeIterator for CsrRangesIterator<'_, From, To> {}

pub struct EnumeratingCsrRangesIterator<'a, From: Index, To: Index> {
    iterator: CsrRangesIterator<'a, From, To>,
}

impl<'a, From: Index, To: Index> Iterator for EnumeratingCsrRangesIterator<'a, From, To> {
    type Item = (From, <CsrRangesIterator<'a, From, To> as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iterator.next_index?;
        let from = From::from_raw(From::RawType::from_usize(index));
        Some((from, self.iterator.next()?))
    }
}
