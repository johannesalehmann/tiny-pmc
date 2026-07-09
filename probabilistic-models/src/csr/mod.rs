mod range;
pub use range::CsrRange;

use crate::Index;
use crate::index::RawIndex;
use std::marker::PhantomData;

#[derive(Default)]
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
        let last_end_index = self.end_to();
        assert_eq!(
            start_to, last_end_index,
            "Entries in `Csr` must be contiguous, i.e. `start_to` must be equal to `self.end_to()`"
        );

        assert_eq!(from.raw().as_usize(), self.entries.len());

        self.entries.push(end_to);
    }

    pub fn extend_last_entry(&mut self, new_to: To) {
        let last_index = self.entries.len() - 1;
        self.entries[last_index] = new_to;
    }

    pub fn end_to(&self) -> To {
        if let Some(entry) = self.entries.last() {
            *entry
        } else {
            To::from_raw(To::RawType::from_usize(0))
        }
    }

    pub fn get(&self, index: From) -> Option<CsrRange<To>> {
        use num_traits::Zero;
        let raw_index = index.raw().as_usize();
        let start = if raw_index > 0 {
            self.entries.get(raw_index - 1)?.raw()
        } else {
            To::RawType::zero()
        };
        let end = self.entries.get(raw_index)?.raw();
        Some(CsrRange {
            start,
            end,
            phantom_data: PhantomData,
        })
    }
}

impl<'a, From: Index, To: Index> IntoIterator for &'a Csr<From, To> {
    type Item = CsrRange<To>;
    type IntoIter = CsrIterator<'a, From, To>;

    fn into_iter(self) -> Self::IntoIter {
        CsrIterator {
            csr: self,
            next_index: Some(0),
        }
    }
}

pub struct CsrIterator<'a, From: Index, To: Index> {
    csr: &'a Csr<From, To>,
    next_index: Option<usize>,
}

impl<'a, From: Index, To: Index> CsrIterator<'a, From, To> {
    pub fn enumerate(self) -> EnumeratingCsrIterator<'a, From, To> {
        EnumeratingCsrIterator { iterator: self }
    }
}

impl<'a, From: Index, To: Index> Iterator for CsrIterator<'a, From, To> {
    type Item = CsrRange<To>;

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

impl<From: Index, To: Index> ExactSizeIterator for CsrIterator<'_, From, To> {}

pub struct EnumeratingCsrIterator<'a, From: Index, To: Index> {
    iterator: CsrIterator<'a, From, To>,
}

impl<'a, From: Index, To: Index> Iterator for EnumeratingCsrIterator<'a, From, To> {
    type Item = (From, <CsrIterator<'a, From, To> as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iterator.next_index?;
        let from = From::from_raw(From::RawType::from_usize(index));
        Some((from, self.iterator.next()?))
    }
}

#[cfg(test)]
mod test {
    use crate::csr::{Csr, CsrRange};
    use crate::{BranchIndex, Index, StateIndex};
    use std::marker::PhantomData;

    macro_rules! check_csr {
        ($test_name: ident, $($index: expr),*) => {
            #[test]
            fn $test_name() {
                let csr: Csr<StateIndex<u32>, BranchIndex<u32>> = Csr::with_entries(vec![
                    $(BranchIndex::from_raw($index)),*
                ]);
                let mut iter = csr.into_iter();
                let mut enum_iter = csr.into_iter().enumerate();
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
            let (index, mut enum_entry) = $enum_iter.next().unwrap();
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
                CsrRange {
                    start: to_start,
                    end: to_end,
                    phantom_data: PhantomData
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
