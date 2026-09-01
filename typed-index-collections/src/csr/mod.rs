pub mod chain;
mod range;

pub use range::{CsrRangeIterator, CsrRanges, CsrRangesIterator, EnumeratingCsrRangesIterator};

use crate::index::RawIndex;
use crate::{Index, IndexRange, SemiboundedIndexRange};
use std::marker::PhantomData;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Csr<From: Index, To: Index> {
    entries: Vec<To>,
    phantom_data: PhantomData<From>,
}

impl<From: Index, To: Index> Csr<From, To> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            phantom_data: PhantomData,
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            phantom_data: PhantomData,
        }
    }

    pub fn with_entries<V: Into<Vec<To>>>(entries: V) -> Self {
        Self {
            entries: entries.into(),
            phantom_data: PhantomData,
        }
    }

    pub fn add_entry(&mut self, from: From, start_to: To, end_to: To) {
        let last_end_index = self.end();
        assert_eq!(
            start_to, last_end_index,
            "Entries in `Csr` must be contiguous, i.e. `start_to` must be equal to `self.end_to()`"
        );

        assert_eq!(from.raw().as_usize(), self.entries.len());

        self.entries.push(end_to);
    }

    // TODO: Unify between To1 and Csr how checked and unchecked adding works
    pub fn add_entry_unchecked(&mut self, to: To) {
        self.entries.push(to);
    }

    pub fn extend_last_entry(&mut self, new_to: To) {
        if self.entries.len() == 0 {
            panic!("Cannot extend last entry of `Csr` without entries.")
        }
        let last_index = self.entries.len() - 1;
        self.entries[last_index] = new_to;
    }

    // TODO: Offer `.values()` function instead, then this function can be replaced by
    //  .values().end()
    pub fn end(&self) -> To {
        if let Some(entry) = self.entries.last() {
            *entry
        } else {
            To::from_raw(To::RawType::from_usize(0))
        }
    }

    pub fn get(&self, index: From) -> Option<IndexRange<To>> {
        let raw_index = index.raw().as_usize();
        let start = if raw_index > 0 {
            *self.entries.get(raw_index - 1)?
        } else {
            To::from_raw(To::RawType::zero())
        };
        let end = *self.entries.get(raw_index)?;
        Some(IndexRange::new(start, end))
    }

    pub fn index(&self, index: From) -> IndexRange<To> {
        self.get(index).unwrap()
    }

    pub fn keys(&self) -> SemiboundedIndexRange<From> {
        SemiboundedIndexRange::new(From::from_raw(From::RawType::from_usize(
            self.entries.len(),
        )))
    }

    pub fn values(&self) -> SemiboundedIndexRange<To> {
        let length = match self.entries.last() {
            Some(last) => *last,
            None => To::from_raw(To::RawType::zero()),
        };
        SemiboundedIndexRange::new(length)
    }

    pub fn ranges(&self) -> CsrRanges<'_, From, To> {
        CsrRanges::new(self)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<'a, From: Index, To: Index> IntoIterator for &'a Csr<From, To> {
    type Item = (From, To);
    type IntoIter = CsrIterator<'a, From, To>;

    fn into_iter(self) -> Self::IntoIter {
        CsrIterator {
            csr: self,
            from_index: 0,
            to_index: To::from_raw(To::RawType::zero()),
            end: self.values().end(),
        }
    }
}

pub struct CsrIterator<'a, From: Index, To: Index> {
    csr: &'a Csr<From, To>,
    from_index: usize,
    to_index: To,
    end: To,
}

impl<'a, From: Index, To: Index> Iterator for CsrIterator<'a, From, To> {
    type Item = (From, To);

    fn next(&mut self) -> Option<Self::Item> {
        if self.to_index.raw() < self.end.raw() {
            while self.csr.entries[self.from_index].raw() <= self.to_index.raw() {
                self.from_index += 1;
            }
            let res = (
                From::from_raw(From::RawType::from_usize(self.from_index)),
                self.to_index,
            );
            self.to_index += To::RawType::one();
            Some(res)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate as typed_index_collections;
    use crate::csr::Csr;
    use crate::{Index, IndexRange};

    crate::index!(BranchIndex);
    crate::index!(StateIndex);

    macro_rules! check_csr {
        ($test_name: ident, $($index: expr),*) => {
            #[test]
            fn $test_name() {
                let csr: Csr<StateIndex<u32>, BranchIndex<u32>> = Csr::with_entries(vec![
                    $(BranchIndex::from_raw($index)),*
                ]);
                let mut iter = csr.ranges().into_iter();
                let mut enum_iter = csr.ranges().into_iter().enumerate();
                let indices = &[0, $($index),*];
                for (from, (&start, &end)) in (indices.iter().zip(indices[1..].iter())).enumerate() {
                    check_iters!(iter, enum_iter, from, start, end);
                }
                assert_eq!(iter.next(), None);
                assert_eq!(enum_iter.next(), None);
            }
        };
    }

    macro_rules! check_iters {
        ($iter: ident, $enum_iter: ident, $from: expr, $to_start: expr, $to_end: expr) => {{
            let mut entry = $iter.next().unwrap();
            let (index, enum_entry) = $enum_iter.next().unwrap();
            assert_eq!(
                index,
                StateIndex::from_raw($from as u32),
                "`From` index of enumerating iterator was incorrect"
            );
            check_entry!(entry, $to_start, $to_end);
            check_entry!(enum_entry, $to_start, $to_end);
        }};
    }

    macro_rules! check_entry {
        ($entry: expr, $to_start: expr, $to_end: expr) => {
            let to_start: u32 = $to_start;
            let to_end: u32 = $to_end;
            assert_eq!(
                $entry,
                IndexRange {
                    start: BranchIndex::from_raw(to_start),
                    end: BranchIndex::from_raw(to_end),
                }
            );
            let mut iter = $entry.into_iter();
            println!("{} to {}:", to_start, to_end);
            for i in to_start..to_end {
                println!("  {}", i);
                assert_eq!(iter.next(), Some(BranchIndex::from_raw(i)));
            }
            assert_eq!(iter.next(), None);
        };
    }

    check_csr!(empty,);
    check_csr!(only_zero, 0);
    check_csr!(repeated_zero, 0, 0, 0);
    check_csr!(singletons, 1, 2, 3, 4, 5);
    check_csr!(different_sized_entries, 1, 3, 5, 12, 31);
    check_csr!(zero_at_start, 0, 0, 5, 12);
    check_csr!(some_entries_zero, 4, 5, 8, 8, 8, 12, 14, 21, 21);
}
