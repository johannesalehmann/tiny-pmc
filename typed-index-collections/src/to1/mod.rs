use crate::index::RawIndex;
use crate::{Index, SemiboundedIndexRange};
use std::marker::PhantomData;
use std::ops::Range;

pub struct To1<From: Index, E> {
    entries: Vec<E>,
    _phantom_data: PhantomData<From>,
}

impl<From: Index + Default, E> Default for To1<From, E> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            _phantom_data: PhantomData,
        }
    }
}

impl<From: Index, E> To1<From, E> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            _phantom_data: PhantomData,
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            _phantom_data: PhantomData,
        }
    }

    pub fn with_entries<Es: Into<Vec<E>>>(entries: Es) -> Self {
        Self {
            entries: entries.into(),
            _phantom_data: PhantomData,
        }
    }

    pub fn keys(&self) -> SemiboundedIndexRange<From> {
        SemiboundedIndexRange::new(From::from_raw(From::RawType::from_usize(
            self.entries.len(),
        )))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, index: From) -> Option<&E> {
        self.entries.get(index.raw().as_usize())
    }

    pub fn get_mut(&mut self, index: From) -> Option<&mut E> {
        self.entries.get_mut(index.raw().as_usize())
    }

    pub fn add_checked(&mut self, index: From, element: E) -> From {
        assert_eq!(index.raw().as_usize(), self.entries.len());
        self.entries.push(element);
        index
    }

    pub fn add(&mut self, element: E) -> From {
        let index = From::from_raw(From::RawType::from_usize(self.entries.len()));
        self.entries.push(element);
        index
    }

    pub fn take(mut self, index: From) -> Option<E> {
        let raw = index.raw().as_usize();
        if raw < self.entries.len() {
            Some(self.entries.swap_remove(raw))
        } else {
            None
        }
    }

    pub fn map<E2>(&self, map: impl FnMut(&E) -> E2) -> To1<From, E2> {
        To1 {
            entries: self.entries.iter().map(map).collect::<Vec<_>>(),
            _phantom_data: PhantomData,
        }
    }

    pub fn change_key_type<From2: Index>(self) -> To1<From2, E> {
        To1 {
            entries: self.entries,
            _phantom_data: PhantomData,
        }
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn enumerate(&self) -> EnumeratingTo1Iterator<'_, From, E> {
        EnumeratingTo1Iterator {
            iterator: self.iter(),
            index: From::RawType::zero(),
        }
    }
}

impl<From: Index> To1<From, bool> {
    pub fn true_values(&self) -> To1BoolValues<'_, From> {
        To1BoolValues {
            condition: true,
            to1: self,
        }
    }
    pub fn false_values(&self) -> To1BoolValues<'_, From> {
        To1BoolValues {
            condition: false,
            to1: self,
        }
    }
}

pub struct To1BoolValues<'a, From: Index> {
    condition: bool,
    to1: &'a To1<From, bool>,
}

impl<'a, From: Index> IntoIterator for &To1BoolValues<'a, From> {
    type Item = From;
    type IntoIter = To1BoolValuesIterator<'a, From>;

    fn into_iter(self) -> Self::IntoIter {
        To1BoolValuesIterator {
            condition: self.condition,
            to1: self.to1,
            index: From::from_raw(From::RawType::zero()),
        }
    }
}

pub struct To1BoolValuesIterator<'a, From: Index> {
    condition: bool,
    to1: &'a To1<From, bool>,
    index: From,
}

impl<'a, From: Index> Iterator for To1BoolValuesIterator<'a, From> {
    type Item = From;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.to1.keys().end() {
            if self.to1[self.index] == self.condition {
                let res = Some(self.index);
                self.index += From::RawType::one();
                return res;
            } else {
                self.index += From::RawType::one();
            }
        }
        None
    }
}

impl<From: Index, E> std::ops::Index<From> for To1<From, E> {
    type Output = E;

    fn index(&self, index: From) -> &Self::Output {
        &self.entries[index.raw().as_usize()]
    }
}

impl<From: Index, E> std::ops::IndexMut<From> for To1<From, E> {
    fn index_mut(&mut self, index: From) -> &mut Self::Output {
        &mut self.entries[index.raw().as_usize()]
    }
}

impl<From: Index, E> std::ops::Index<Range<From>> for To1<From, E> {
    type Output = [E];

    fn index(&self, index: Range<From>) -> &Self::Output {
        &self.entries[index.start.raw().as_usize()..index.end.raw().as_usize()]
    }
}

impl<From: Index, E> std::ops::IndexMut<Range<From>> for To1<From, E> {
    fn index_mut(&mut self, index: Range<From>) -> &mut Self::Output {
        &mut self.entries[index.start.raw().as_usize()..index.end.raw().as_usize()]
    }
}

impl<'a, From: Index, E> IntoIterator for &'a To1<From, E> {
    type Item = &'a E;
    type IntoIter = std::slice::Iter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<'a, From: Index, E> IntoIterator for &'a mut To1<From, E> {
    type Item = &'a mut E;
    type IntoIter = std::slice::IterMut<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter_mut()
    }
}

impl<From: Index, E> IntoIterator for To1<From, E> {
    type Item = E;
    type IntoIter = std::vec::IntoIter<E>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

pub struct EnumeratingTo1Iterator<'a, From: Index, E> {
    iterator: <&'a To1<From, E> as IntoIterator>::IntoIter,
    index: From::RawType,
}

impl<'a, From: Index, E> Iterator for EnumeratingTo1Iterator<'a, From, E> {
    type Item = (From, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|val| {
            let index = self.index;
            self.index = self.index + From::RawType::one();
            (From::from_raw(index), val)
        })
    }
}
