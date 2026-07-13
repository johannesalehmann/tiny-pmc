use crate::{Csr, Index, RawIndex};

#[derive(Copy, Clone)]
pub struct SemiboundedIndexRange<Idx: Index> {
    end: Idx,
}

impl<Idx: Index> SemiboundedIndexRange<Idx> {
    pub fn new(end: Idx) -> Self {
        Self { end }
    }

    pub fn len(&self) -> usize {
        self.end.raw().as_usize()
    }

    pub fn end(&self) -> Idx {
        self.end
    }

    pub fn index(&self, index: usize) -> Idx {
        if index >= self.end.raw().as_usize() {
            panic!("Out-of-range indexing operation into `SemiboundedIndexRange`");
        }
        Idx::from_raw(Idx::RawType::from_usize(index))
    }
    pub fn get(&self, index: usize) -> Option<Idx> {
        if index >= self.end.raw().as_usize() {
            None
        } else {
            Some(Idx::from_raw(Idx::RawType::from_usize(index)))
        }
    }

    pub fn last(&self) -> Option<Idx> {
        if self.end.raw() == Idx::RawType::zero() {
            None
        } else {
            Some(self.end - Idx::RawType::one())
        }
    }
}

impl<Idx: Index> IntoIterator for SemiboundedIndexRange<Idx> {
    type Item = Idx;
    type IntoIter = IndexRangeIterator<Idx>;

    fn into_iter(self) -> Self::IntoIter {
        IndexRangeIterator {
            next: Idx::from_raw(Idx::RawType::zero()),
            length: self.end,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IndexRange<Idx: Index> {
    pub start: Idx,
    pub end: Idx,
}

impl<Idx: Index> IndexRange<Idx> {
    pub fn new(start: Idx, end: Idx) -> Self {
        Self { start, end }
    }

    pub fn with_single_index(index: Idx) -> Self {
        Self {
            start: index,
            end: index + Idx::RawType::one(),
        }
    }

    pub fn len(&self) -> usize {
        (self.end.raw() - self.start.raw()).as_usize()
    }

    pub fn index(&self, index: usize) -> Idx {
        let offset = self.start.raw().as_usize();
        if offset + index >= self.end.raw().as_usize() {
            panic!("Out-of-range indexing operation into `SemiboundedIndexRange`");
        }
        Idx::from_raw(Idx::RawType::from_usize(offset + index))
    }
    pub fn get(&self, index: usize) -> Option<Idx> {
        let offset = self.start.raw().as_usize();
        if offset + index >= self.end.raw().as_usize() {
            None
        } else {
            Some(Idx::from_raw(Idx::RawType::from_usize(offset + index)))
        }
    }
}

impl<Idx: Index> From<SemiboundedIndexRange<Idx>> for IndexRange<Idx> {
    fn from(value: SemiboundedIndexRange<Idx>) -> Self {
        Self {
            start: Idx::from_raw(Idx::RawType::zero()),
            end: value.end,
        }
    }
}

impl<Idx: Index> IntoIterator for IndexRange<Idx> {
    type Item = Idx;
    type IntoIter = IndexRangeIterator<Idx>;

    fn into_iter(self) -> Self::IntoIter {
        IndexRangeIterator {
            next: self.start,
            length: self.end,
        }
    }
}

pub struct IndexRangeIterator<Idx: Index> {
    next: Idx,
    length: Idx,
}

impl<Idx: Index> Iterator for IndexRangeIterator<Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.raw() < self.length.raw() {
            let res = self.next;
            self.next += Idx::RawType::one();
            Some(res)
        } else {
            None
        }
    }
}
