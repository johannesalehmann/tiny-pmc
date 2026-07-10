use crate::Index;
use crate::index::RawIndex;
use num_integer::Integer;

pub struct CsrRangeIterator<To: Index> {
    pub(super) from: To::RawType,
    pub(super) to: To::RawType,
    pub(super) next_entry: To::RawType,
}

impl<To: Index> Iterator for CsrRangeIterator<To> {
    type Item = To;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_entry < self.to {
            let res = Some(To::from_raw(self.next_entry));
            self.next_entry.inc();
            res
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size: To::RawType = self.to - self.next_entry;
        (size.as_usize(), Some(size.as_usize()))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let size: To::RawType = self.to - self.next_entry;
        size.as_usize()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        if self.next_entry < self.to {
            Some(To::from_raw(self.to - To::RawType::one()))
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.next_entry = self.next_entry + To::RawType::from_usize(n);
        if self.next_entry > self.to {
            self.next_entry = self.to;
        }
        self.next()
    }
}

impl<To: Index> ExactSizeIterator for CsrRangeIterator<To> {}
