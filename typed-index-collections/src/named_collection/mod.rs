use crate::to1::To1;
use crate::{Index, RawIndex, SemiboundedIndexRange, ValuePerIndexSource};
use std::collections::HashMap;

pub struct NamedTo1<InternalIndex: Index, E> {
    store: To1<InternalIndex, E>,
    names: To1<InternalIndex, String>,
    name_to_index: HashMap<String, InternalIndex>,
}

// We cannot derive Default for NamedTo1, because it should implement default even if E does not.
impl<InternalIndex: Index, E> Default for NamedTo1<InternalIndex, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<InternalIndex: Index, E> NamedTo1<InternalIndex, E> {
    pub fn new() -> Self {
        Self {
            store: To1::new(),
            names: To1::new(),
            name_to_index: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, name: String, entry: E) -> InternalIndex {
        if self.name_to_index.contains_key(&name) {
            panic!("Cannot add a second entry with name `{name}` to this `NamedTo1` collection.")
        }
        let index = self.store.add(entry);
        self.name_to_index.insert(name.clone(), index);
        self.names.add_checked(index, name);
        index
    }

    pub fn get(&self, index: InternalIndex) -> Option<&E> {
        self.store.get(index)
    }

    pub fn get_mut(&mut self, index: InternalIndex) -> Option<&mut E> {
        self.store.get_mut(index)
    }

    pub fn entry_by_name(&self, name: &str) -> Option<&E> {
        let index = self.name_to_index.get(name)?;
        self.store.get(*index)
    }

    pub fn entry_by_name_mut(&mut self, name: &str) -> Option<&mut E> {
        let index = self.name_to_index.get(name)?;
        self.store.get_mut(*index)
    }

    pub fn index_by_name(&self, name: &str) -> Option<InternalIndex> {
        self.name_to_index.get(name).cloned()
    }

    pub fn name(&self, index: InternalIndex) -> Option<&str> {
        self.names.get(index).map(String::as_str)
    }

    pub fn names(&self) -> &To1<InternalIndex, String> {
        &self.names
    }

    pub fn internal_indices(&self) -> SemiboundedIndexRange<InternalIndex> {
        self.store.keys().change_index_type()
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn entries(&self) -> &To1<InternalIndex, E> {
        &self.store
    }
}

impl<InternalIndex: Index, E> std::ops::Index<InternalIndex> for NamedTo1<InternalIndex, E> {
    type Output = E;

    fn index(&self, index: InternalIndex) -> &Self::Output {
        &self.store[index]
    }
}

impl<InternalIndex: Index, E> std::ops::IndexMut<InternalIndex> for NamedTo1<InternalIndex, E> {
    fn index_mut(&mut self, index: InternalIndex) -> &mut Self::Output {
        &mut self.store[index]
    }
}

impl<InternalIndex: Index, E> std::ops::Index<&str> for NamedTo1<InternalIndex, E> {
    type Output = E;

    fn index(&self, name: &str) -> &Self::Output {
        let index = self
            .name_to_index
            .get(name)
            .expect("This `NamedTo1` collection contains no  entry with name `{name}`.");
        &self.store[*index]
    }
}

impl<InternalIndex: Index, E> std::ops::IndexMut<&str> for NamedTo1<InternalIndex, E> {
    fn index_mut(&mut self, name: &str) -> &mut Self::Output {
        let index = self
            .name_to_index
            .get(name)
            .expect("This `NamedTo1` collection contains no  entry with name `{name}`.");
        &mut self.store[*index]
    }
}

impl<'a, InternalIndex: Index, E> IntoIterator for &'a NamedTo1<InternalIndex, E> {
    type Item = (&'a str, &'a E);
    type IntoIter = NamedTo1Iterator<'a, InternalIndex, E>;

    fn into_iter(self) -> Self::IntoIter {
        NamedTo1Iterator {
            named_to_1: self,
            index: InternalIndex::from_raw(InternalIndex::RawType::zero()),
        }
    }
}

pub struct NamedTo1Iterator<'a, InternalIndex: Index, E> {
    named_to_1: &'a NamedTo1<InternalIndex, E>,
    index: InternalIndex,
}

impl<'a, InternalIndex: Index, E> Iterator for NamedTo1Iterator<'a, InternalIndex, E> {
    type Item = (&'a str, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.named_to_1.store.keys().end() {
            let res = Some((
                self.named_to_1.name(self.index).unwrap(),
                &self.named_to_1[self.index],
            ));
            self.index += InternalIndex::RawType::one();
            res
        } else {
            None
        }
    }
}
